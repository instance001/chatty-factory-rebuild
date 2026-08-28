#[cfg(windows)]
mod windows_cli {
    use std::process::Command;

    use chatty_factory_rebuild::{
        BuildPlan, BuildStepSpec, CommandBuildSessionConfig, CommandJsonProviderConfig,
        ExactRequest, JsonProviderTranscript,
    };

    #[test]
    fn cli_runs_command_provider_session_from_files() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let provider_script = runtime.path().join("cli-provider.ps1");
        std::fs::write(
            &provider_script,
            r#"
$ErrorActionPreference = 'Stop'
$raw = [Console]::In.ReadToEnd()
$request = $raw | ConvertFrom-Json
$stepId = $request.step.step_id
$failureCount = @($request.prior_failures).Count
if ($stepId -eq 'readme') {
  $proposal = [ordered]@{
    proposal_id = 'cli-command-proposal-readme'
    summary = 'CLI external command supplied README method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'README.md'; contents = "CLI command demo`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main' -and $failureCount -eq 0) {
  $proposal = [ordered]@{
    proposal_id = 'cli-command-proposal-main-wrong-first'
    summary = 'CLI external command supplied first main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "wrong`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main') {
  $proposal = [ordered]@{
    proposal_id = 'cli-command-proposal-main-repaired'
    summary = 'CLI external command supplied repaired main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "cli-ready`n" } })
    suggested_verification = @()
  }
} else {
  throw "unexpected step $stepId"
}
$proposal | ConvertTo-Json -Depth 10 -Compress
"#,
        )
        .unwrap();
        let plan = BuildPlan::new(
            "cli-session-plan",
            ExactRequest::new(
                "request-cli-session",
                "Build a CLI command-backed session demo.",
            ),
            vec![
                BuildStepSpec::new(
                    "readme",
                    "Create the README artifact.",
                    vec!["file_contains:README.md::CLI command demo".to_string()],
                ),
                BuildStepSpec::new(
                    "main",
                    "Create the main text artifact.",
                    vec!["file_contains:src/main.txt::cli-ready".to_string()],
                ),
            ],
        )
        .unwrap();
        let plan_path = runtime.path().join("plans/cli-plan.json");
        plan.write_json_file(&plan_path).unwrap();
        let provider_config_path = runtime.path().join("providers/cli-provider.json");
        CommandJsonProviderConfig::new(
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-File".to_string(),
                provider_script.display().to_string(),
            ],
            None,
        )
        .write_json_file(&provider_config_path)
        .unwrap();
        let manifest_path = runtime.path().join("cli/build-manifest.json");
        let bundle_root = runtime.path().join("cli/bundle");
        let session_config_path = runtime.path().join("sessions/cli-session.json");
        let session_config = CommandBuildSessionConfig::new(
            runtime.path(),
            workspace.path(),
            &plan_path,
            &provider_config_path,
            "trace-cli-session",
            "request-cli-session",
            "operator-confirmed-cli-session-config-build",
            4,
            4096,
            2,
            Some(manifest_path.clone()),
            Some(bundle_root.clone()),
            None,
        )
        .unwrap();
        session_config
            .write_json_file(&session_config_path)
            .unwrap();
        assert_eq!(
            CommandBuildSessionConfig::read_json_file(&session_config_path).unwrap(),
            session_config
        );

        let output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("--session-config")
            .arg(&session_config_path)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("complete plan=cli-session-plan"));
        assert!(stdout.contains("steps=2"));
        assert!(stdout.contains("transcript_entries=3"));
        assert_eq!(
            std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
            "CLI command demo\n"
        );
        assert_eq!(
            std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
            "cli-ready\n"
        );
        assert!(manifest_path.exists());
        assert_eq!(
            std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
            "cli-ready\n"
        );
        assert!(bundle_root
            .join("evidence/provider_transcript.json")
            .exists());

        let status_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("status")
            .arg("--session-config")
            .arg(&session_config_path)
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            status_output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&status_output.stdout),
            String::from_utf8_lossy(&status_output.stderr)
        );
        let status_json: serde_json::Value = serde_json::from_slice(&status_output.stdout).unwrap();
        assert_eq!(status_json["plan_id"], "cli-session-plan");
        assert_eq!(status_json["complete"], true);
        assert_eq!(status_json["completed_steps"].as_array().unwrap().len(), 2);

        let history_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("history")
            .arg("--session-config")
            .arg(&session_config_path)
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            history_output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&history_output.stdout),
            String::from_utf8_lossy(&history_output.stderr)
        );
        let history_json: serde_json::Value =
            serde_json::from_slice(&history_output.stdout).unwrap();
        assert_eq!(
            history_json
                .as_array()
                .unwrap()
                .iter()
                .filter(|record| record["record_kind"] == "Proposal")
                .count(),
            3
        );
        assert!(history_json.as_array().unwrap().iter().any(|record| {
            record["record_kind"] == "FailureEvidence"
                && record["failure_class"] == "VerificationFailed"
        }));

        let verify_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("verify-bundle")
            .arg("--bundle")
            .arg(&bundle_root)
            .output()
            .unwrap();
        assert!(
            verify_output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&verify_output.stdout),
            String::from_utf8_lossy(&verify_output.stderr)
        );
        let verify_stdout = String::from_utf8(verify_output.stdout).unwrap();
        assert!(verify_stdout.contains("bundle-verified plan=cli-session-plan"));
        assert!(verify_stdout.contains("complete=true"));
        assert!(verify_stdout.contains("artifacts=2"));
        assert!(verify_stdout.contains("transcript_present=true"));

        std::fs::write(
            bundle_root.join("artifacts/src/main.txt"),
            "tampered from cli test\n",
        )
        .unwrap();
        let tampered_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("verify-bundle")
            .arg("--bundle")
            .arg(&bundle_root)
            .output()
            .unwrap();
        assert!(!tampered_output.status.success());
        assert!(
            String::from_utf8_lossy(&tampered_output.stderr).contains("does not match manifest")
        );
    }

    #[test]
    fn cli_rerun_resumes_from_journal_evidence_without_redoing_completed_steps() {
        let workspace = tempfile::tempdir().unwrap();
        let runtime = tempfile::tempdir().unwrap();
        let calls_log = runtime.path().join("provider-calls.log");
        let provider_script = runtime.path().join("cli-resume-provider.ps1");
        let calls_log_literal = calls_log.display().to_string().replace('\'', "''");
        std::fs::write(
            &provider_script,
            r#"
$ErrorActionPreference = 'Stop'
$raw = [Console]::In.ReadToEnd()
$request = $raw | ConvertFrom-Json
$stepId = $request.step.step_id
$failureCount = @($request.prior_failures).Count
Add-Content -LiteralPath '__CALLS_LOG__' -Value "$stepId`:$failureCount"
if ($stepId -eq 'readme') {
  $proposal = [ordered]@{
    proposal_id = 'cli-resume-proposal-readme'
    summary = 'CLI resume external command supplied README method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'README.md'; contents = "CLI resume demo`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main' -and $failureCount -eq 0) {
  $proposal = [ordered]@{
    proposal_id = 'cli-resume-proposal-main-wrong-first'
    summary = 'CLI resume external command supplied first main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "wrong`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main' -and $failureCount -eq 1) {
  $proposal = [ordered]@{
    proposal_id = 'cli-resume-proposal-main-repaired'
    summary = 'CLI resume external command supplied repaired main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "resume-ready`n" } })
    suggested_verification = @()
  }
} else {
  throw "unexpected step/failure count $stepId/$failureCount"
}
$proposal | ConvertTo-Json -Depth 10 -Compress
"#
            .replace("__CALLS_LOG__", &calls_log_literal),
        )
        .unwrap();
        let plan = BuildPlan::new(
            "cli-resume-plan",
            ExactRequest::new(
                "request-cli-resume",
                "Build a CLI session that can resume after failure evidence.",
            ),
            vec![
                BuildStepSpec::new(
                    "readme",
                    "Create the README artifact.",
                    vec!["file_contains:README.md::CLI resume demo".to_string()],
                ),
                BuildStepSpec::new(
                    "main",
                    "Create the main text artifact.",
                    vec!["file_contains:src/main.txt::resume-ready".to_string()],
                ),
            ],
        )
        .unwrap();
        let plan_path = runtime.path().join("plans/cli-resume-plan.json");
        plan.write_json_file(&plan_path).unwrap();
        let provider_config_path = runtime.path().join("providers/cli-resume-provider.json");
        CommandJsonProviderConfig::new(
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-File".to_string(),
                provider_script.display().to_string(),
            ],
            None,
        )
        .write_json_file(&provider_config_path)
        .unwrap();
        let manifest_path = runtime.path().join("cli-resume/build-manifest.json");
        let bundle_root = runtime.path().join("cli-resume/bundle");
        let first_transcript_path = runtime.path().join("cli-resume/first-transcript.json");
        let second_transcript_path = runtime.path().join("cli-resume/second-transcript.json");

        let first_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--provider-config")
            .arg(&provider_config_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .arg("--max-attempts-per-step")
            .arg("1")
            .arg("--manifest")
            .arg(&manifest_path)
            .arg("--bundle")
            .arg(&bundle_root)
            .arg("--transcript")
            .arg(&first_transcript_path)
            .output()
            .unwrap();

        assert!(
            first_output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&first_output.stdout),
            String::from_utf8_lossy(&first_output.stderr)
        );
        let first_stdout = String::from_utf8(first_output.stdout).unwrap();
        assert!(first_stdout.contains("stopped plan=cli-resume-plan"));
        assert!(first_stdout.contains("step=main"));
        assert!(first_stdout.contains("reason=step attempt limit reached"));
        assert!(first_stdout.contains("transcript_entries=2"));
        assert_eq!(
            std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
            "CLI resume demo\n"
        );
        assert_eq!(
            std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
            "wrong\n"
        );
        assert!(!bundle_root.exists());
        let first_transcript =
            JsonProviderTranscript::read_json_file(&first_transcript_path).unwrap();
        assert_eq!(first_transcript.entries().len(), 2);
        assert!(first_transcript.entries()[1]
            .response_json()
            .unwrap()
            .contains("cli-resume-proposal-main-wrong-first"));

        let stopped_status = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("status")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .output()
            .unwrap();
        assert!(
            stopped_status.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&stopped_status.stdout),
            String::from_utf8_lossy(&stopped_status.stderr)
        );
        let stopped_status_stdout = String::from_utf8(stopped_status.stdout).unwrap();
        assert!(stopped_status_stdout.contains(
            "status plan=cli-resume-plan complete=false completed_steps=1 current_step=main current_failures=1"
        ));
        assert!(stopped_status_stdout.contains("completed step=readme attempts=1 artifacts=1"));
        assert!(
            stopped_status_stdout.contains("failure step=main attempt=1 class=VerificationFailed")
        );

        let stopped_json_status = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("status")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            stopped_json_status.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&stopped_json_status.stdout),
            String::from_utf8_lossy(&stopped_json_status.stderr)
        );
        let stopped_json: serde_json::Value =
            serde_json::from_slice(&stopped_json_status.stdout).unwrap();
        assert_eq!(stopped_json["plan_id"], "cli-resume-plan");
        assert_eq!(stopped_json["complete"], false);
        assert_eq!(stopped_json["current_step_id"], "main");
        assert_eq!(stopped_json["completed_steps"].as_array().unwrap().len(), 1);
        assert_eq!(
            stopped_json["current_failures"][0]["failure_class"],
            "VerificationFailed"
        );

        let stopped_history = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("history")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .output()
            .unwrap();
        assert!(
            stopped_history.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&stopped_history.stdout),
            String::from_utf8_lossy(&stopped_history.stderr)
        );
        let stopped_history_stdout = String::from_utf8(stopped_history.stdout).unwrap();
        assert!(stopped_history_stdout.contains("history records="));
        assert!(stopped_history_stdout.contains("proposal=cli-resume-proposal-readme"));
        assert!(stopped_history_stdout.contains("proposal=cli-resume-proposal-main-wrong-first"));
        assert!(stopped_history_stdout.contains("kind=FailureEvidence"));
        assert!(stopped_history_stdout.contains("failure_class=VerificationFailed"));
        assert_eq!(
            std::fs::read_to_string(&calls_log)
                .unwrap()
                .replace("\r\n", "\n"),
            "readme:0\nmain:0\n"
        );

        let second_output = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--provider-config")
            .arg(&provider_config_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .arg("--max-attempts-per-step")
            .arg("2")
            .arg("--manifest")
            .arg(&manifest_path)
            .arg("--bundle")
            .arg(&bundle_root)
            .arg("--transcript")
            .arg(&second_transcript_path)
            .output()
            .unwrap();

        assert!(
            second_output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&second_output.stdout),
            String::from_utf8_lossy(&second_output.stderr)
        );
        let second_stdout = String::from_utf8(second_output.stdout).unwrap();
        assert!(second_stdout.contains("complete plan=cli-resume-plan"));
        assert!(second_stdout.contains("steps=2"));
        assert!(second_stdout.contains("transcript_entries=1"));
        assert_eq!(
            std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
            "resume-ready\n"
        );
        assert_eq!(
            std::fs::read_to_string(&calls_log)
                .unwrap()
                .replace("\r\n", "\n"),
            "readme:0\nmain:0\nmain:1\n"
        );
        assert_eq!(
            std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
            "resume-ready\n"
        );
        let second_transcript =
            JsonProviderTranscript::read_json_file(&second_transcript_path).unwrap();
        assert_eq!(second_transcript.entries().len(), 1);
        assert_eq!(
            second_transcript.entries()[0].request().step().step_id(),
            "main"
        );
        assert_eq!(
            second_transcript.entries()[0]
                .request()
                .prior_failures()
                .len(),
            1
        );
        assert!(second_transcript.entries()[0]
            .response_json()
            .unwrap()
            .contains("cli-resume-proposal-main-repaired"));
        assert!(bundle_root
            .join("evidence/provider_transcript.json")
            .exists());

        let complete_status = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("status")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .output()
            .unwrap();
        assert!(
            complete_status.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&complete_status.stdout),
            String::from_utf8_lossy(&complete_status.stderr)
        );
        let complete_status_stdout = String::from_utf8(complete_status.stdout).unwrap();
        assert!(complete_status_stdout.contains(
            "status plan=cli-resume-plan complete=true completed_steps=2 current_step=none current_failures=0"
        ));
        assert!(complete_status_stdout.contains("completed step=readme attempts=1 artifacts=1"));
        assert!(complete_status_stdout.contains("completed step=main attempts=2 artifacts=1"));

        let complete_json_status = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("status")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--plan")
            .arg(&plan_path)
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            complete_json_status.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&complete_json_status.stdout),
            String::from_utf8_lossy(&complete_json_status.stderr)
        );
        let complete_json: serde_json::Value =
            serde_json::from_slice(&complete_json_status.stdout).unwrap();
        assert_eq!(complete_json["plan_id"], "cli-resume-plan");
        assert_eq!(complete_json["complete"], true);
        assert!(complete_json["current_step_id"].is_null());
        assert_eq!(
            complete_json["completed_steps"].as_array().unwrap().len(),
            2
        );
        assert_eq!(complete_json["completed_steps"][1]["step_id"], "main");
        assert_eq!(complete_json["completed_steps"][1]["attempts_used"], 2);
        assert_eq!(
            complete_json["current_failures"].as_array().unwrap().len(),
            0
        );

        let complete_json_history = Command::new(env!("CARGO_BIN_EXE_chatty-factory-rebuild"))
            .arg("history")
            .arg("--runtime-root")
            .arg(runtime.path())
            .arg("--workspace-root")
            .arg(workspace.path())
            .arg("--trace-id")
            .arg("trace-cli-resume")
            .arg("--request-id")
            .arg("request-cli-resume")
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            complete_json_history.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&complete_json_history.stdout),
            String::from_utf8_lossy(&complete_json_history.stderr)
        );
        let complete_history: serde_json::Value =
            serde_json::from_slice(&complete_json_history.stdout).unwrap();
        let history_records = complete_history.as_array().unwrap();
        assert_eq!(
            history_records
                .iter()
                .filter(|record| record["record_kind"] == "Proposal")
                .count(),
            3
        );
        assert_eq!(
            history_records
                .iter()
                .filter(|record| record["record_kind"] == "FailureEvidence")
                .count(),
            1
        );
        assert!(history_records.iter().any(|record| {
            record["record_kind"] == "Proposal"
                && record["proposal_id"] == "cli-resume-proposal-main-repaired"
        }));
        assert!(history_records.iter().any(|record| {
            record["record_kind"] == "VerificationReceipt" && record["success"] == false
        }));
        assert!(history_records.iter().any(|record| {
            record["record_kind"] == "VerificationReceipt" && record["success"] == true
        }));
        assert_eq!(
            std::fs::read_to_string(&calls_log)
                .unwrap()
                .replace("\r\n", "\n"),
            "readme:0\nmain:0\nmain:1\n"
        );
    }
}
