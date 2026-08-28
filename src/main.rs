use std::env;
use std::path::PathBuf;

use chatty_factory_rebuild::{
    BuildBundle, BuildPlan, BuildRunOutcome, CommandBuildSessionConfig, ExecutionReceipt,
    FailureEvidenceReceipt, GateReceipt, HostBounds, MethodProposal, RuntimeJournal,
    RuntimeRecordEnvelope, RuntimeRecordKind, VerificationReceipt, WorkOrderReceipt,
};
use serde::Serialize;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if raw_args.first().map(String::as_str) == Some("verify-bundle") {
        return run_verify_bundle(&raw_args[1..]);
    }
    if raw_args.first().map(String::as_str) == Some("status") {
        return run_status(&raw_args[1..]);
    }
    if raw_args.first().map(String::as_str) == Some("history") {
        return run_history(&raw_args[1..]);
    }
    if raw_args.first().map(String::as_str) == Some("--session-config") {
        return run_session_config(&raw_args[1..]);
    }
    run_command_session(CliArgs::parse(raw_args)?.into_config()?)
}

fn run_session_config(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err(format!(
            "expected exactly one --session-config path\n{}",
            usage()
        ));
    }
    run_command_session(CommandBuildSessionConfig::read_json_file(&args[0])?)
}

fn run_command_session(args: CommandBuildSessionConfig) -> Result<(), String> {
    let journal = RuntimeJournal::new(
        args.runtime_root(),
        args.trace_id(),
        args.request_id(),
        args.host_bounds(),
    );
    let limits = args.limits()?;
    let exports = args.exports();
    let result = journal.run_command_json_build_session_from_files(
        args.plan_path(),
        args.provider_config_path(),
        &limits,
        args.confirmation_context(),
        &exports,
    )?;
    match result.session().outcome() {
        BuildRunOutcome::Complete { .. } => {
            println!(
                "complete plan={} steps={} journal_records={} transcript_entries={}",
                result.session().report().plan_id(),
                result.session().report().completed_steps().len(),
                result.session().report().journal_record_count(),
                result.provider_transcript().entries().len()
            );
            if let Some(manifest) = result.session().manifest() {
                println!("manifest_schema={}", manifest.schema_version());
            }
            if let Some(bundle) = result.session().bundle() {
                println!("bundle={}", bundle.bundle_root().display());
            }
        }
        BuildRunOutcome::Stopped {
            stopped_step_id,
            reason,
            ..
        } => {
            println!(
                "stopped plan={} step={} reason={} transcript_entries={}",
                result.session().report().plan_id(),
                stopped_step_id,
                reason,
                result.provider_transcript().entries().len()
            );
        }
        _ => {
            println!(
                "unknown-outcome plan={} transcript_entries={}",
                result.session().report().plan_id(),
                result.provider_transcript().entries().len()
            );
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct HistoryRecordSummary {
    sequence_number: u64,
    record_id: String,
    record_kind: String,
    attempt_id: Option<String>,
    proposal_id: Option<String>,
    admissible: Option<bool>,
    success: Option<bool>,
    failure_class: Option<String>,
    summary: String,
}

fn run_history(args: &[String]) -> Result<(), String> {
    let args = HistoryArgs::parse(args.to_vec())?;
    let journal = RuntimeJournal::new(
        args.runtime_root(),
        args.trace_id(),
        args.request_id(),
        args.host_bounds(),
    );
    let summaries = journal
        .verify()?
        .iter()
        .map(history_summary)
        .collect::<Result<Vec<_>, _>>()?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summaries)
                .map_err(|err| format!("could not serialize build history: {err}"))?
        );
        return Ok(());
    }
    println!("history records={}", summaries.len());
    for summary in summaries {
        println!(
            "record seq={} kind={} attempt={} proposal={} admissible={} success={} failure_class={} summary={}",
            summary.sequence_number,
            summary.record_kind,
            summary.attempt_id.as_deref().unwrap_or("none"),
            summary.proposal_id.as_deref().unwrap_or("none"),
            summary
                .admissible
                .map(|value| value.to_string())
                .as_deref()
                .unwrap_or("none"),
            summary
                .success
                .map(|value| value.to_string())
                .as_deref()
                .unwrap_or("none"),
            summary.failure_class.as_deref().unwrap_or("none"),
            summary.summary
        );
    }
    Ok(())
}

fn history_summary(record: &RuntimeRecordEnvelope) -> Result<HistoryRecordSummary, String> {
    let mut summary = HistoryRecordSummary {
        sequence_number: record.sequence_number(),
        record_id: record.record_id().to_string(),
        record_kind: format!("{:?}", record.record_kind()),
        attempt_id: record.attempt_id().map(ToOwned::to_owned),
        proposal_id: None,
        admissible: None,
        success: None,
        failure_class: None,
        summary: format!("{:?}", record.record_kind()),
    };
    match record.record_kind() {
        RuntimeRecordKind::Proposal => {
            let proposal: MethodProposal = serde_json::from_value(record.payload().clone())
                .map_err(|err| format!("could not decode proposal history record: {err}"))?;
            summary.proposal_id = Some(proposal.proposal_id().to_string());
            summary.summary = proposal.summary().to_string();
        }
        RuntimeRecordKind::GateReceipt => {
            let gate: GateReceipt = serde_json::from_value(record.payload().clone())
                .map_err(|err| format!("could not decode gate history record: {err}"))?;
            summary.proposal_id = Some(gate.proposal_id().to_string());
            summary.admissible = Some(gate.admissible());
            summary.summary = if gate.admissible() {
                "gate allowed proposal".to_string()
            } else {
                format!("gate rejected proposal: {}", gate.reasons().join("; "))
            };
        }
        RuntimeRecordKind::WorkOrderReceipt => {
            let work_order: WorkOrderReceipt = serde_json::from_value(record.payload().clone())
                .map_err(|err| format!("could not decode work-order history record: {err}"))?;
            summary.proposal_id = Some(work_order.proposal_id().to_string());
            summary.summary = format!(
                "work order with {} bounded step(s)",
                work_order.steps().len()
            );
        }
        RuntimeRecordKind::ExecutionReceipt => {
            let execution: ExecutionReceipt = serde_json::from_value(record.payload().clone())
                .map_err(|err| format!("could not decode execution history record: {err}"))?;
            summary.summary = format!(
                "executed {} step(s), wrote {} file(s)",
                execution.executed_steps(),
                execution.written_files().len()
            );
        }
        RuntimeRecordKind::VerificationReceipt => {
            let verification: VerificationReceipt =
                serde_json::from_value(record.payload().clone()).map_err(|err| {
                    format!("could not decode verification history record: {err}")
                })?;
            summary.success = Some(verification.success());
            summary.summary = format!(
                "verification {} with {} evidence entrie(s)",
                if verification.success() {
                    "succeeded"
                } else {
                    "failed"
                },
                verification.evidence().len()
            );
        }
        RuntimeRecordKind::FailureEvidence => {
            let failure: FailureEvidenceReceipt = serde_json::from_value(record.payload().clone())
                .map_err(|err| format!("could not decode failure history record: {err}"))?;
            summary.failure_class = Some(format!("{:?}", failure.failure_class()));
            summary.summary = format!(
                "failure evidence with {} evidence entrie(s) and {} lock signal(s)",
                failure.evidence().len(),
                failure.lock_signals().len()
            );
        }
        _ => {}
    }
    Ok(summary)
}

fn run_status(args: &[String]) -> Result<(), String> {
    let args = StatusArgs::parse(args.to_vec())?;
    let journal = RuntimeJournal::new(
        args.runtime_root(),
        args.trace_id(),
        args.request_id(),
        args.host_bounds(),
    );
    let plan = BuildPlan::read_json_file(args.plan_path())?;
    let report = journal.build_report(&plan)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|err| format!("could not serialize build status: {err}"))?
        );
        return Ok(());
    }
    println!(
        "status plan={} complete={} completed_steps={} current_step={} current_failures={} journal_records={}",
        report.plan_id(),
        report.complete(),
        report.completed_steps().len(),
        report.current_step_id().unwrap_or("none"),
        report.current_failures().len(),
        report.journal_record_count()
    );
    for step in report.completed_steps() {
        println!(
            "completed step={} attempts={} artifacts={} execution_record={} verification_record={}",
            step.step_id(),
            step.attempts_used(),
            step.artifacts().len(),
            step.execution_record_id(),
            step.verification_record_id()
        );
    }
    for failure in report.current_failures() {
        println!(
            "failure step={} attempt={} class={:?} evidence_entries={} lock_signals={}",
            failure.step_id(),
            failure.attempt_number(),
            failure.failure_class(),
            failure.evidence().len(),
            failure.lock_signals().len()
        );
    }
    Ok(())
}

fn run_verify_bundle(args: &[String]) -> Result<(), String> {
    let mut parser = FlagParser {
        args: args.to_vec(),
        index: 0,
    };
    let mut bundle_root = PathBuf::new();
    while let Some(flag) = parser.next_flag()? {
        match flag.as_str() {
            "--bundle" => bundle_root = PathBuf::from(parser.value(&flag)?),
            _ => return Err(format!("unknown argument '{flag}'\n{}", usage())),
        }
    }
    if bundle_root.as_os_str().is_empty() {
        return Err(format!("missing --bundle\n{}", usage()));
    }
    let bundle = BuildBundle::read_verified(&bundle_root)?;
    println!(
        "bundle-verified plan={} complete={} artifacts={} transcript_present={}",
        bundle.manifest().report().plan_id(),
        bundle.manifest().report().complete(),
        bundle.copied_artifacts().len(),
        bundle.provider_transcript().is_some()
    );
    Ok(())
}

struct HistoryArgs {
    runtime_root: PathBuf,
    workspace_root: PathBuf,
    trace_id: String,
    request_id: String,
    max_files: usize,
    max_file_bytes: usize,
    json: bool,
}

impl HistoryArgs {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            return Err(usage());
        }
        if args.first().map(String::as_str) == Some("--session-config") {
            if args.len() != 2 && args.len() != 3 {
                return Err(format!(
                    "expected --session-config PATH [--json]\n{}",
                    usage()
                ));
            }
            let config = CommandBuildSessionConfig::read_json_file(&args[1])?;
            let json = args.get(2).map(String::as_str) == Some("--json");
            if args.len() == 3 && !json {
                return Err(format!("unknown argument '{}'\n{}", args[2], usage()));
            }
            return Ok(Self::from_config(config, json));
        }
        let mut parser = FlagParser { args, index: 0 };
        let mut parsed = Self {
            runtime_root: PathBuf::new(),
            workspace_root: PathBuf::new(),
            trace_id: String::new(),
            request_id: String::new(),
            max_files: 64,
            max_file_bytes: 1024 * 1024,
            json: false,
        };
        while let Some(flag) = parser.next_flag()? {
            match flag.as_str() {
                "--runtime-root" => parsed.runtime_root = PathBuf::from(parser.value(&flag)?),
                "--workspace-root" => parsed.workspace_root = PathBuf::from(parser.value(&flag)?),
                "--trace-id" => parsed.trace_id = parser.value(&flag)?,
                "--request-id" => parsed.request_id = parser.value(&flag)?,
                "--max-files" => parsed.max_files = parse_usize(&flag, parser.value(&flag)?)?,
                "--max-file-bytes" => {
                    parsed.max_file_bytes = parse_usize(&flag, parser.value(&flag)?)?
                }
                "--json" => parsed.json = true,
                _ => return Err(format!("unknown argument '{flag}'\n{}", usage())),
            }
        }
        parsed.validate()?;
        Ok(parsed)
    }

    fn from_config(config: CommandBuildSessionConfig, json: bool) -> Self {
        Self {
            runtime_root: config.runtime_root().to_path_buf(),
            workspace_root: config.workspace_root().to_path_buf(),
            trace_id: config.trace_id().to_string(),
            request_id: config.request_id().to_string(),
            max_files: config.max_files(),
            max_file_bytes: config.max_file_bytes(),
            json,
        }
    }

    fn runtime_root(&self) -> &std::path::Path {
        &self.runtime_root
    }

    fn trace_id(&self) -> &str {
        &self.trace_id
    }

    fn request_id(&self) -> &str {
        &self.request_id
    }

    fn host_bounds(&self) -> HostBounds {
        HostBounds::new(
            self.workspace_root.clone(),
            self.max_files,
            self.max_file_bytes,
        )
    }

    fn validate(&self) -> Result<(), String> {
        if self.runtime_root.as_os_str().is_empty() {
            return Err(format!("missing --runtime-root\n{}", usage()));
        }
        if self.workspace_root.as_os_str().is_empty() {
            return Err(format!("missing --workspace-root\n{}", usage()));
        }
        if self.trace_id.trim().is_empty() {
            return Err(format!("missing --trace-id\n{}", usage()));
        }
        if self.request_id.trim().is_empty() {
            return Err(format!("missing --request-id\n{}", usage()));
        }
        Ok(())
    }
}

struct StatusArgs {
    runtime_root: PathBuf,
    workspace_root: PathBuf,
    plan_path: PathBuf,
    trace_id: String,
    request_id: String,
    max_files: usize,
    max_file_bytes: usize,
    json: bool,
}

impl StatusArgs {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            return Err(usage());
        }
        if args.first().map(String::as_str) == Some("--session-config") {
            if args.len() != 2 && args.len() != 3 {
                return Err(format!(
                    "expected --session-config PATH [--json]\n{}",
                    usage()
                ));
            }
            let config = CommandBuildSessionConfig::read_json_file(&args[1])?;
            let json = args.get(2).map(String::as_str) == Some("--json");
            if args.len() == 3 && !json {
                return Err(format!("unknown argument '{}'\n{}", args[2], usage()));
            }
            return Ok(Self::from_config(config, json));
        }
        let mut parser = FlagParser { args, index: 0 };
        let mut parsed = Self {
            runtime_root: PathBuf::new(),
            workspace_root: PathBuf::new(),
            plan_path: PathBuf::new(),
            trace_id: String::new(),
            request_id: String::new(),
            max_files: 64,
            max_file_bytes: 1024 * 1024,
            json: false,
        };
        while let Some(flag) = parser.next_flag()? {
            match flag.as_str() {
                "--runtime-root" => parsed.runtime_root = PathBuf::from(parser.value(&flag)?),
                "--workspace-root" => parsed.workspace_root = PathBuf::from(parser.value(&flag)?),
                "--plan" => parsed.plan_path = PathBuf::from(parser.value(&flag)?),
                "--trace-id" => parsed.trace_id = parser.value(&flag)?,
                "--request-id" => parsed.request_id = parser.value(&flag)?,
                "--max-files" => parsed.max_files = parse_usize(&flag, parser.value(&flag)?)?,
                "--max-file-bytes" => {
                    parsed.max_file_bytes = parse_usize(&flag, parser.value(&flag)?)?
                }
                "--json" => parsed.json = true,
                _ => return Err(format!("unknown argument '{flag}'\n{}", usage())),
            }
        }
        parsed.validate()?;
        Ok(parsed)
    }

    fn from_config(config: CommandBuildSessionConfig, json: bool) -> Self {
        Self {
            runtime_root: config.runtime_root().to_path_buf(),
            workspace_root: config.workspace_root().to_path_buf(),
            plan_path: config.plan_path().to_path_buf(),
            trace_id: config.trace_id().to_string(),
            request_id: config.request_id().to_string(),
            max_files: config.max_files(),
            max_file_bytes: config.max_file_bytes(),
            json,
        }
    }

    fn runtime_root(&self) -> &std::path::Path {
        &self.runtime_root
    }

    fn plan_path(&self) -> &std::path::Path {
        &self.plan_path
    }

    fn trace_id(&self) -> &str {
        &self.trace_id
    }

    fn request_id(&self) -> &str {
        &self.request_id
    }

    fn host_bounds(&self) -> HostBounds {
        HostBounds::new(
            self.workspace_root.clone(),
            self.max_files,
            self.max_file_bytes,
        )
    }

    fn validate(&self) -> Result<(), String> {
        if self.runtime_root.as_os_str().is_empty() {
            return Err(format!("missing --runtime-root\n{}", usage()));
        }
        if self.workspace_root.as_os_str().is_empty() {
            return Err(format!("missing --workspace-root\n{}", usage()));
        }
        if self.plan_path.as_os_str().is_empty() {
            return Err(format!("missing --plan\n{}", usage()));
        }
        if self.trace_id.trim().is_empty() {
            return Err(format!("missing --trace-id\n{}", usage()));
        }
        if self.request_id.trim().is_empty() {
            return Err(format!("missing --request-id\n{}", usage()));
        }
        Ok(())
    }
}

struct CliArgs {
    runtime_root: PathBuf,
    workspace_root: PathBuf,
    plan_path: PathBuf,
    provider_config_path: PathBuf,
    trace_id: String,
    request_id: String,
    confirmation_context: String,
    max_files: usize,
    max_file_bytes: usize,
    max_attempts_per_step: usize,
    manifest_path: Option<PathBuf>,
    bundle_root: Option<PathBuf>,
    transcript_path: Option<PathBuf>,
}

impl CliArgs {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            return Err(usage());
        }
        let mut parser = FlagParser { args, index: 0 };
        let mut parsed = Self {
            runtime_root: PathBuf::new(),
            workspace_root: PathBuf::new(),
            plan_path: PathBuf::new(),
            provider_config_path: PathBuf::new(),
            trace_id: String::new(),
            request_id: String::new(),
            confirmation_context: "external-operator-confirmed-build".to_string(),
            max_files: 64,
            max_file_bytes: 1024 * 1024,
            max_attempts_per_step: 3,
            manifest_path: None,
            bundle_root: None,
            transcript_path: None,
        };
        while let Some(flag) = parser.next_flag()? {
            match flag.as_str() {
                "--runtime-root" => parsed.runtime_root = PathBuf::from(parser.value(&flag)?),
                "--workspace-root" => parsed.workspace_root = PathBuf::from(parser.value(&flag)?),
                "--plan" => parsed.plan_path = PathBuf::from(parser.value(&flag)?),
                "--provider-config" => {
                    parsed.provider_config_path = PathBuf::from(parser.value(&flag)?)
                }
                "--trace-id" => parsed.trace_id = parser.value(&flag)?,
                "--request-id" => parsed.request_id = parser.value(&flag)?,
                "--confirmation-context" => parsed.confirmation_context = parser.value(&flag)?,
                "--max-files" => parsed.max_files = parse_usize(&flag, parser.value(&flag)?)?,
                "--max-file-bytes" => {
                    parsed.max_file_bytes = parse_usize(&flag, parser.value(&flag)?)?
                }
                "--max-attempts-per-step" => {
                    parsed.max_attempts_per_step = parse_usize(&flag, parser.value(&flag)?)?
                }
                "--manifest" => parsed.manifest_path = Some(PathBuf::from(parser.value(&flag)?)),
                "--bundle" => parsed.bundle_root = Some(PathBuf::from(parser.value(&flag)?)),
                "--transcript" => {
                    parsed.transcript_path = Some(PathBuf::from(parser.value(&flag)?))
                }
                _ => return Err(format!("unknown argument '{flag}'\n{}", usage())),
            }
        }
        parsed.validate()?;
        Ok(parsed)
    }

    fn into_config(self) -> Result<CommandBuildSessionConfig, String> {
        CommandBuildSessionConfig::new(
            self.runtime_root,
            self.workspace_root,
            self.plan_path,
            self.provider_config_path,
            self.trace_id,
            self.request_id,
            self.confirmation_context,
            self.max_files,
            self.max_file_bytes,
            self.max_attempts_per_step,
            self.manifest_path,
            self.bundle_root,
            self.transcript_path,
        )
    }

    fn validate(&self) -> Result<(), String> {
        if self.runtime_root.as_os_str().is_empty() {
            return Err(format!("missing --runtime-root\n{}", usage()));
        }
        if self.workspace_root.as_os_str().is_empty() {
            return Err(format!("missing --workspace-root\n{}", usage()));
        }
        if self.plan_path.as_os_str().is_empty() {
            return Err(format!("missing --plan\n{}", usage()));
        }
        if self.provider_config_path.as_os_str().is_empty() {
            return Err(format!("missing --provider-config\n{}", usage()));
        }
        if self.trace_id.trim().is_empty() {
            return Err(format!("missing --trace-id\n{}", usage()));
        }
        if self.request_id.trim().is_empty() {
            return Err(format!("missing --request-id\n{}", usage()));
        }
        Ok(())
    }
}

struct FlagParser {
    args: Vec<String>,
    index: usize,
}

impl FlagParser {
    fn next_flag(&mut self) -> Result<Option<String>, String> {
        if self.index >= self.args.len() {
            return Ok(None);
        }
        let flag = self.args[self.index].clone();
        self.index += 1;
        if !flag.starts_with("--") {
            return Err(format!("expected flag, got '{}'\n{}", flag, usage()));
        }
        Ok(Some(flag))
    }

    fn value(&mut self, flag: &str) -> Result<String, String> {
        if self.index >= self.args.len() {
            return Err(format!("missing value for '{flag}'"));
        }
        let value = self.args[self.index].clone();
        self.index += 1;
        if value.starts_with("--") {
            return Err(format!("missing value for '{flag}'"));
        }
        Ok(value)
    }
}

fn parse_usize(flag: &str, value: String) -> Result<usize, String> {
    value
        .parse()
        .map_err(|err| format!("invalid value for '{flag}': {err}"))
}

fn usage() -> String {
    "usage: chatty-factory-rebuild --session-config PATH\n       chatty-factory-rebuild --runtime-root PATH --workspace-root PATH --plan PATH --provider-config PATH --trace-id ID --request-id ID [--confirmation-context TEXT] [--max-files N] [--max-file-bytes N] [--max-attempts-per-step N] [--manifest PATH] [--bundle PATH] [--transcript PATH]\n       chatty-factory-rebuild status --session-config PATH [--json]\n       chatty-factory-rebuild status --runtime-root PATH --workspace-root PATH --plan PATH --trace-id ID --request-id ID [--max-files N] [--max-file-bytes N] [--json]\n       chatty-factory-rebuild history --session-config PATH [--json]\n       chatty-factory-rebuild history --runtime-root PATH --workspace-root PATH --trace-id ID --request-id ID [--max-files N] [--max-file-bytes N] [--json]\n       chatty-factory-rebuild verify-bundle --bundle PATH".to_string()
}
