use std::path::PathBuf;
use std::time::Duration;

use chatty_factory_rebuild::{
    external_operator_assertion, BuildBundle, BuildFailureContext, BuildPlan, BuildProposalContext,
    BuildProposalProvider, BuildProposalRequest, BuildRunLimits, BuildRunOutcome,
    BuildSessionExportPaths, BuildStepSpec, CommandJsonProviderConfig, EfRescueAttemptOutcome,
    ExactRequest, HostBounds, IntentClaim, IntentClaimKind, IntentDraft, JsonBuildProposalProvider,
    JsonBuildProposalService, JsonProviderTranscript, MethodProposal, ProposedStep, RuntimeJournal,
    RuntimeRecordKind, SourceSpan, TranscriptJsonBuildProposalService,
};

#[test]
fn public_factory_api_builds_a_verified_artifact() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let request_text = "Create README with hello";
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-public",
        "request-public",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let intent = journal
        .confirm_intent(
            IntentDraft::new(
                "draft-public",
                ExactRequest::new("request-public", request_text),
                vec![IntentClaim::new(
                    "accept-readme",
                    IntentClaimKind::AcceptanceCriterion,
                    "file_contains:README.md::hello",
                    vec![SourceSpan::new(0, request_text.len(), request_text)],
                )],
            ),
            external_operator_assertion("operator-confirmed-public-build"),
        )
        .unwrap();
    let proposal = MethodProposal::new(
        "proposal-public-write-readme",
        "externally supplied method: write the requested README",
        vec![ProposedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello from the public factory".to_string(),
        }],
        vec!["model suggested check is inert".to_string()],
    );

    let outcome = journal.run_ef_rescue_attempt(&intent, &proposal).unwrap();

    let EfRescueAttemptOutcome::Artifact {
        execution,
        verification,
        ..
    } = outcome
    else {
        panic!("public factory path should produce a verified artifact");
    };
    assert_eq!(execution.written_files(), &[PathBuf::from("README.md")]);
    assert!(verification.success());
    assert_eq!(
        verification.checked_claim_ids(),
        &["accept-readme".to_string()]
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "hello from the public factory"
    );
    assert!(journal.verify().is_ok());
}

struct DemoProvider;

impl BuildProposalProvider for DemoProvider {
    fn propose(&mut self, context: &BuildProposalContext) -> Result<MethodProposal, String> {
        match context.step().step_id() {
            "readme" => Ok(MethodProposal::new(
                "proposal-readme",
                "externally supplied method for README",
                vec![ProposedStep::WriteFile {
                    path: PathBuf::from("README.md"),
                    contents: "ChattyFactory demo\n".to_string(),
                }],
                vec!["suggested verification remains inert".to_string()],
            )),
            "main" if context.prior_failures().is_empty() => Ok(MethodProposal::new(
                "proposal-main-wrong-first",
                "externally supplied method that misses host acceptance",
                vec![ProposedStep::WriteFile {
                    path: PathBuf::from("src/main.txt"),
                    contents: "not ready yet".to_string(),
                }],
                vec!["model says this is fine".to_string()],
            )),
            "main" => {
                assert_eq!(context.prior_failures().len(), 1);
                assert_host_failure_feedback(context.prior_failures());
                Ok(MethodProposal::new(
                    "proposal-main-repaired-from-evidence",
                    "externally supplied repaired method after host evidence",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("src/main.txt"),
                        contents: "factory-ready\n".to_string(),
                    }],
                    vec![],
                ))
            }
            step_id => Err(format!("unexpected step {step_id}")),
        }
    }
}

struct StopThenResumeProvider {
    allow_repair: bool,
    readme_calls: usize,
    main_calls: usize,
}

struct ProviderFailureThenResumeProvider {
    fail_main: bool,
    readme_calls: usize,
    main_calls: usize,
}

struct JsonDemoService {
    requests: Vec<BuildProposalRequest>,
}

impl JsonBuildProposalService for JsonDemoService {
    fn propose_json(&mut self, request_json: &str) -> Result<String, String> {
        let request: BuildProposalRequest =
            serde_json::from_str(request_json).map_err(|err| err.to_string())?;
        let proposal = match request.step().step_id() {
            "readme" => {
                assert!(request.prior_failures().is_empty());
                MethodProposal::new(
                    "json-proposal-readme",
                    "JSON-supplied README method",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("README.md"),
                        contents: "JSON factory demo\n".to_string(),
                    }],
                    vec!["json provider suggested check remains inert".to_string()],
                )
            }
            "main" if request.prior_failures().is_empty() => MethodProposal::new(
                "json-proposal-main-wrong-first",
                "JSON-supplied first main method",
                vec![ProposedStep::WriteFile {
                    path: PathBuf::from("src/main.txt"),
                    contents: "wrong".to_string(),
                }],
                vec![],
            ),
            "main" => {
                assert_eq!(request.prior_failures().len(), 1);
                assert!(request.prior_failures()[0]
                    .evidence()
                    .iter()
                    .any(|entry| entry.contains("main-accept-1")));
                MethodProposal::new(
                    "json-proposal-main-repaired",
                    "JSON-supplied repaired main method",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("src/main.txt"),
                        contents: "json-ready\n".to_string(),
                    }],
                    vec![],
                )
            }
            step_id => return Err(format!("unexpected step {step_id}")),
        };
        self.requests.push(request);
        serde_json::to_string(&proposal).map_err(|err| err.to_string())
    }
}

impl BuildProposalProvider for ProviderFailureThenResumeProvider {
    fn propose(&mut self, context: &BuildProposalContext) -> Result<MethodProposal, String> {
        match context.step().step_id() {
            "readme" => {
                self.readme_calls += 1;
                Ok(MethodProposal::new(
                    "proposal-readme-provider-failure-demo",
                    "externally supplied README method",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("README.md"),
                        contents: "Resume demo\n".to_string(),
                    }],
                    vec![],
                ))
            }
            "main" if self.fail_main => {
                self.main_calls += 1;
                Err("provider unavailable for main".to_string())
            }
            "main" => {
                self.main_calls += 1;
                assert!(context.prior_failures().is_empty());
                Ok(MethodProposal::new(
                    "proposal-main-after-provider-resume",
                    "externally supplied method after provider resumes",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("src/main.txt"),
                        contents: "resumed-ok\n".to_string(),
                    }],
                    vec![],
                ))
            }
            step_id => Err(format!("unexpected step {step_id}")),
        }
    }
}

impl BuildProposalProvider for StopThenResumeProvider {
    fn propose(&mut self, context: &BuildProposalContext) -> Result<MethodProposal, String> {
        match context.step().step_id() {
            "readme" => {
                self.readme_calls += 1;
                Ok(MethodProposal::new(
                    "proposal-readme-resume-demo",
                    "externally supplied README method",
                    vec![ProposedStep::WriteFile {
                        path: PathBuf::from("README.md"),
                        contents: "Resume demo\n".to_string(),
                    }],
                    vec![],
                ))
            }
            "main" => {
                self.main_calls += 1;
                if self.allow_repair {
                    assert_eq!(context.prior_failures().len(), 1);
                    assert!(context.prior_failures()[0]
                        .evidence()
                        .iter()
                        .any(|entry| entry.contains("main-accept-1")));
                    Ok(MethodProposal::new(
                        "proposal-main-resumed-repair",
                        "externally supplied repair after resume",
                        vec![ProposedStep::WriteFile {
                            path: PathBuf::from("src/main.txt"),
                            contents: "resumed-ok\n".to_string(),
                        }],
                        vec![],
                    ))
                } else {
                    Ok(MethodProposal::new(
                        "proposal-main-stops-first-run",
                        "externally supplied method that will fail verification",
                        vec![ProposedStep::WriteFile {
                            path: PathBuf::from("src/main.txt"),
                            contents: "still wrong".to_string(),
                        }],
                        vec![],
                    ))
                }
            }
            step_id => Err(format!("unexpected step {step_id}")),
        }
    }
}

fn assert_host_failure_feedback(failures: &[BuildFailureContext]) {
    let failure = &failures[0];
    assert_eq!(failure.step_id(), "main");
    assert_eq!(failure.attempt_number(), 1);
    assert!(failure
        .evidence()
        .iter()
        .any(|entry| entry.contains("claim 'main-accept-1' failed")));
    assert_eq!(failure.lock_signals(), &["write:src/main.txt".to_string()]);
}

fn demo_plan(request_id: &str, request_text: &str) -> BuildPlan {
    BuildPlan::new(
        "demo-plan",
        ExactRequest::new(request_id, request_text),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::Resume demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::resumed-ok".to_string()],
            ),
        ],
    )
    .unwrap()
}

#[test]
fn public_stepped_factory_completes_multistep_build_with_evidence_retry() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let request_text =
        "Build a tiny demo with a README and a main text artifact. Keep the steps deterministic.";
    let plan = BuildPlan::new(
        "demo-plan",
        ExactRequest::new("request-stepped", request_text),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::ChattyFactory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::factory-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-stepped",
        "request-stepped",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let mut provider = DemoProvider;
    let limits = BuildRunLimits::new(2).unwrap();

    let outcome = journal
        .run_stepped_build(
            &plan,
            &mut provider,
            &limits,
            "operator-confirmed-stepped-build",
        )
        .unwrap();

    let BuildRunOutcome::Complete {
        completed_steps,
        journal_records,
    } = outcome
    else {
        panic!("stepped factory should complete after evidence-guided retry");
    };
    assert_eq!(completed_steps.len(), 2);
    assert_eq!(completed_steps[0].step_id(), "readme");
    assert_eq!(completed_steps[0].attempts_used(), 1);
    assert_eq!(completed_steps[1].step_id(), "main");
    assert_eq!(completed_steps[1].attempts_used(), 2);
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "ChattyFactory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "factory-ready\n"
    );
    assert_eq!(
        journal_records
            .iter()
            .filter(|record| record.record_kind() == RuntimeRecordKind::FailureEvidence)
            .count(),
        1
    );
    assert_eq!(
        journal_records
            .iter()
            .filter(|record| record.record_kind() == RuntimeRecordKind::VerificationReceipt)
            .count(),
        3
    );
    assert!(!journal_records
        .iter()
        .any(|record| record.record_kind() == RuntimeRecordKind::PromotionCandidate));
    assert!(journal.verify().is_ok());
}

#[test]
fn public_stepped_factory_resumes_after_explicit_stop_without_redoing_completed_steps() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let request_text = "Build a resumable demo with README and main artifacts.";
    let plan = demo_plan("request-resume", request_text);
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-resume",
        "request-resume",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let mut first_provider = StopThenResumeProvider {
        allow_repair: false,
        readme_calls: 0,
        main_calls: 0,
    };

    let first = journal
        .run_stepped_build(
            &plan,
            &mut first_provider,
            &BuildRunLimits::new(1).unwrap(),
            "operator-confirmed-resume-build",
        )
        .unwrap();

    let BuildRunOutcome::Stopped {
        completed_steps,
        stopped_step_id,
        failures,
        ..
    } = first
    else {
        panic!("first run should stop at the explicit per-step attempt limit");
    };
    assert_eq!(completed_steps.len(), 1);
    assert_eq!(completed_steps[0].step_id(), "readme");
    assert_eq!(stopped_step_id, "main");
    assert_eq!(failures.len(), 1);
    assert_eq!(first_provider.readme_calls, 1);
    assert_eq!(first_provider.main_calls, 1);

    let progress = journal.build_progress(&plan).unwrap();
    assert_eq!(progress.completed_steps().len(), 1);
    assert_eq!(progress.current_step_id(), Some("main"));
    assert_eq!(progress.current_failures().len(), 1);

    let mut second_provider = StopThenResumeProvider {
        allow_repair: true,
        readme_calls: 0,
        main_calls: 0,
    };
    let second = journal
        .run_stepped_build(
            &plan,
            &mut second_provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-resume-build",
        )
        .unwrap();

    let BuildRunOutcome::Complete {
        completed_steps,
        journal_records,
    } = second
    else {
        panic!("second run should resume and complete");
    };
    assert_eq!(second_provider.readme_calls, 0);
    assert_eq!(second_provider.main_calls, 1);
    assert_eq!(completed_steps.len(), 2);
    assert_eq!(completed_steps[0].step_id(), "readme");
    assert_eq!(completed_steps[1].step_id(), "main");
    assert_eq!(completed_steps[1].attempts_used(), 2);
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "Resume demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "resumed-ok\n"
    );
    assert_eq!(
        journal_records
            .iter()
            .filter(|record| record.record_kind() == RuntimeRecordKind::FailureEvidence)
            .count(),
        1
    );
    assert!(journal.build_progress(&plan).unwrap().is_complete());
}

#[test]
fn public_stepped_factory_resumes_after_provider_failure_stop() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let request_text = "Build a provider-failure demo with README and main artifacts.";
    let plan = demo_plan("request-provider-stop", request_text);
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-provider-stop",
        "request-provider-stop",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let mut first_provider = ProviderFailureThenResumeProvider {
        fail_main: true,
        readme_calls: 0,
        main_calls: 0,
    };

    let first = journal
        .run_stepped_build(
            &plan,
            &mut first_provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-provider-stop-build",
        )
        .unwrap();

    let BuildRunOutcome::Stopped {
        completed_steps,
        stopped_step_id,
        reason,
        failures,
        journal_records,
    } = first
    else {
        panic!("provider failure should stop the build");
    };
    assert_eq!(completed_steps.len(), 1);
    assert_eq!(completed_steps[0].step_id(), "readme");
    assert_eq!(stopped_step_id, "main");
    assert!(reason.contains("proposal provider failed"));
    assert!(failures.is_empty());
    assert_eq!(first_provider.readme_calls, 1);
    assert_eq!(first_provider.main_calls, 1);
    assert_eq!(
        journal_records
            .iter()
            .filter(|record| record.record_kind() == RuntimeRecordKind::FailureEvidence)
            .count(),
        0
    );

    let progress = journal.build_progress(&plan).unwrap();
    assert_eq!(progress.completed_steps().len(), 1);
    assert_eq!(progress.current_step_id(), Some("main"));
    assert!(progress.current_failures().is_empty());

    let mut second_provider = ProviderFailureThenResumeProvider {
        fail_main: false,
        readme_calls: 0,
        main_calls: 0,
    };
    let second = journal
        .run_stepped_build(
            &plan,
            &mut second_provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-provider-stop-build",
        )
        .unwrap();

    let BuildRunOutcome::Complete {
        completed_steps, ..
    } = second
    else {
        panic!("provider recovery should resume and complete");
    };
    assert_eq!(second_provider.readme_calls, 0);
    assert_eq!(second_provider.main_calls, 1);
    assert_eq!(completed_steps.len(), 2);
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "resumed-ok\n"
    );
}

#[test]
fn public_stepped_factory_accepts_json_provider_exchange() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let request_text = "Build a JSON-provider demo with README and main artifacts.";
    let plan = BuildPlan::new(
        "json-demo-plan",
        ExactRequest::new("request-json-provider", request_text),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::JSON factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::json-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let plan_json = plan.to_json().unwrap();
    let loaded_plan = BuildPlan::from_json(&plan_json).unwrap();
    assert_eq!(loaded_plan, plan);
    let mut drifted_plan_value: serde_json::Value = serde_json::from_str(&plan_json).unwrap();
    drifted_plan_value["exact_request"]["bytes_sha256"] =
        serde_json::Value::String("not-the-request-hash".to_string());
    assert!(BuildPlan::from_json(&drifted_plan_value.to_string())
        .unwrap_err()
        .contains("request hash does not match text"));
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-json-provider",
        "request-json-provider",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let service = JsonDemoService {
        requests: Vec::new(),
    };
    let mut provider = JsonBuildProposalProvider::new(service);

    let outcome = journal
        .run_stepped_build(
            &loaded_plan,
            &mut provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-json-provider-build",
        )
        .unwrap();

    let BuildRunOutcome::Complete {
        completed_steps,
        journal_records,
    } = outcome
    else {
        panic!("JSON provider exchange should complete after evidence-guided retry");
    };
    let service = provider.into_inner();
    assert_eq!(service.requests.len(), 3);
    assert_eq!(service.requests[0].step().step_id(), "readme");
    assert_eq!(service.requests[1].step().step_id(), "main");
    assert!(service.requests[1].prior_failures().is_empty());
    assert_eq!(service.requests[2].step().step_id(), "main");
    assert_eq!(service.requests[2].prior_failures().len(), 1);
    assert_eq!(completed_steps.len(), 2);
    assert_eq!(completed_steps[1].attempts_used(), 2);
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "JSON factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "json-ready\n"
    );
    assert_eq!(
        journal_records
            .iter()
            .filter(|record| record.record_kind() == RuntimeRecordKind::FailureEvidence)
            .count(),
        1
    );

    let report = journal.build_report(&loaded_plan).unwrap();
    assert!(report.complete());
    assert_eq!(report.plan_id(), "json-demo-plan");
    assert_eq!(report.journal_record_count(), journal_records.len());
    assert!(report.current_step_id().is_none());
    assert!(report.current_failures().is_empty());
    assert_eq!(report.completed_steps().len(), 2);
    assert_eq!(report.completed_steps()[0].step_id(), "readme");
    assert_eq!(
        report.completed_steps()[0].written_files(),
        &[PathBuf::from("README.md")]
    );
    assert_eq!(report.completed_steps()[0].artifacts().len(), 1);
    assert_eq!(
        report.completed_steps()[0].artifacts()[0].path(),
        PathBuf::from("README.md").as_path()
    );
    assert_eq!(report.completed_steps()[0].artifacts()[0].byte_len(), 18);
    assert_eq!(
        report.completed_steps()[0].artifacts()[0].bytes_sha256(),
        "b935b8de6e2269a406fa589206887025154cc7ac9b85f6f76d808a9ce3f41a94"
    );
    assert_eq!(
        report.completed_steps()[0].checked_claim_ids(),
        &["readme-accept-1".to_string()]
    );
    assert_eq!(report.completed_steps()[1].step_id(), "main");
    assert_eq!(report.completed_steps()[1].attempts_used(), 2);
    assert_eq!(
        report.completed_steps()[1].written_files(),
        &[PathBuf::from("src/main.txt")]
    );
    assert_eq!(report.completed_steps()[1].artifacts().len(), 1);
    assert_eq!(
        report.completed_steps()[1].artifacts()[0].path(),
        PathBuf::from("src/main.txt").as_path()
    );
    assert_eq!(report.completed_steps()[1].artifacts()[0].byte_len(), 11);
    assert_eq!(
        report.completed_steps()[1].artifacts()[0].bytes_sha256(),
        "d04664df0de74fe28075462b06cf29cc8811fbd5b6d4d10eec5730db03fbc3fb"
    );
    assert_eq!(
        report.completed_steps()[1].checked_claim_ids(),
        &["main-accept-1".to_string()]
    );
    assert!(report.completed_steps()[1]
        .verification_evidence()
        .iter()
        .any(|entry| entry == "claim 'main-accept-1' passed"));

    let manifest_path = runtime.path().join("exports/build-manifest.json");
    let manifest = journal
        .export_build_manifest(&loaded_plan, &manifest_path)
        .unwrap();
    assert_eq!(manifest.schema_version(), 1);
    assert_eq!(manifest.report(), &report);
    let manifest_json = std::fs::read_to_string(&manifest_path).unwrap();
    assert_eq!(manifest_json, manifest.to_json().unwrap());
    let manifest_value: serde_json::Value = serde_json::from_str(&manifest_json).unwrap();
    assert_eq!(manifest_value["schema_version"], 1);
    assert_eq!(manifest_value["report"]["plan_id"], "json-demo-plan");
    assert_eq!(manifest_value["report"]["complete"], true);
    assert_eq!(
        manifest_value["report"]["completed_steps"][0]["artifacts"][0]["bytes_sha256"],
        "b935b8de6e2269a406fa589206887025154cc7ac9b85f6f76d808a9ce3f41a94"
    );
    assert_eq!(
        manifest_value["report"]["completed_steps"][1]["artifacts"][0]["path"],
        "src/main.txt"
    );
    assert_eq!(
        manifest_value["report"]["completed_steps"][1]["artifacts"][0]["bytes_sha256"],
        "d04664df0de74fe28075462b06cf29cc8811fbd5b6d4d10eec5730db03fbc3fb"
    );

    let bundle_root = runtime.path().join("bundle");
    let bundle = journal
        .export_build_bundle(&loaded_plan, &bundle_root)
        .unwrap();
    assert_eq!(bundle.bundle_root(), bundle_root.as_path());
    assert_eq!(
        bundle.manifest_path(),
        bundle_root.join("build-manifest.json").as_path()
    );
    assert_eq!(bundle.manifest(), &manifest);
    assert_eq!(bundle.copied_artifacts().len(), 2);
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("build-manifest.json")).unwrap(),
        manifest_json
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/README.md")).unwrap(),
        "JSON factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
        "json-ready\n"
    );
    assert_eq!(
        bundle.copied_artifacts()[0].bytes_sha256(),
        "b935b8de6e2269a406fa589206887025154cc7ac9b85f6f76d808a9ce3f41a94"
    );
    assert_eq!(
        bundle.copied_artifacts()[1].bytes_sha256(),
        "d04664df0de74fe28075462b06cf29cc8811fbd5b6d4d10eec5730db03fbc3fb"
    );
    assert_eq!(
        bundle.journal_path(),
        bundle_root.join("evidence/runtime_records.jsonl").as_path()
    );
    assert_eq!(
        bundle.head_anchor_path(),
        bundle_root.join("evidence/journal_head.json").as_path()
    );
    let bundled_journal = std::fs::read_to_string(bundle.journal_path()).unwrap();
    let bundled_lines = bundled_journal.lines().collect::<Vec<_>>();
    assert_eq!(bundled_lines.len(), journal_records.len());
    assert!(bundled_lines[0].contains("\"record_kind\":\"Request\""));
    assert!(bundled_lines
        .last()
        .unwrap()
        .contains(journal_records.last().unwrap().record_hash()));
    let bundled_head: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(bundle.head_anchor_path()).unwrap()).unwrap();
    assert_eq!(bundled_head["expected_record_count"], journal_records.len());
    assert_eq!(
        bundled_head["final_record_id"],
        journal_records.last().unwrap().record_id()
    );
    assert_eq!(
        bundled_head["final_record_hash"],
        journal_records.last().unwrap().record_hash()
    );

    let verified_bundle = BuildBundle::read_verified(&bundle_root).unwrap();
    assert_eq!(verified_bundle.bundle_root(), bundle_root.as_path());
    assert_eq!(verified_bundle.manifest(), &manifest);
    assert_eq!(
        verified_bundle.copied_artifacts(),
        bundle.copied_artifacts()
    );

    let original_bundled_journal = std::fs::read_to_string(bundle.journal_path()).unwrap();
    let mut reordered_lines = original_bundled_journal.lines().collect::<Vec<_>>();
    reordered_lines.swap(0, 1);
    std::fs::write(
        bundle.journal_path(),
        format!("{}\n", reordered_lines.join("\n")),
    )
    .unwrap();
    assert!(BuildBundle::read_verified(&bundle_root)
        .unwrap_err()
        .contains("reordered sequence"));
    std::fs::write(bundle.journal_path(), &original_bundled_journal).unwrap();

    let mut modified_lines = original_bundled_journal.lines().collect::<Vec<_>>();
    let mut modified_record: serde_json::Value = serde_json::from_str(modified_lines[0]).unwrap();
    modified_record["payload"]["text"] = serde_json::Value::String("tampered request".to_string());
    let modified_record_json = modified_record.to_string();
    modified_lines[0] = &modified_record_json;
    std::fs::write(
        bundle.journal_path(),
        format!("{}\n", modified_lines.join("\n")),
    )
    .unwrap();
    assert!(BuildBundle::read_verified(&bundle_root)
        .unwrap_err()
        .contains("payload was modified"));
    std::fs::write(bundle.journal_path(), &original_bundled_journal).unwrap();

    let mut truncated_lines = original_bundled_journal.lines().collect::<Vec<_>>();
    truncated_lines.pop();
    std::fs::write(
        bundle.journal_path(),
        format!("{}\n", truncated_lines.join("\n")),
    )
    .unwrap();
    assert!(BuildBundle::read_verified(&bundle_root)
        .unwrap_err()
        .contains("journal does not match local head anchor"));
    std::fs::write(bundle.journal_path(), &original_bundled_journal).unwrap();

    std::fs::write(
        bundle_root.join("artifacts/src/main.txt"),
        "tampered artifact\n",
    )
    .unwrap();
    assert!(BuildBundle::read_verified(&bundle_root)
        .unwrap_err()
        .contains("does not match manifest"));
}

#[test]
fn public_build_session_orchestrates_json_plan_provider_report_and_bundle() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let plan = BuildPlan::new(
        "session-demo-plan",
        ExactRequest::new(
            "request-session-demo",
            "Build a session demo with README and main artifacts.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::JSON factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::json-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let loaded_plan = BuildPlan::from_json(&plan.to_json().unwrap()).unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-session-demo",
        "request-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let transcript_service = TranscriptJsonBuildProposalService::new(JsonDemoService {
        requests: Vec::new(),
    });
    let mut provider = JsonBuildProposalProvider::new(transcript_service);
    let manifest_path = runtime.path().join("session/build-manifest.json");
    let bundle_root = runtime.path().join("session/bundle");
    let exports = BuildSessionExportPaths::manifest_and_bundle(&manifest_path, &bundle_root);

    let result = journal
        .run_build_session(
            &loaded_plan,
            &mut provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-session-build",
            &exports,
        )
        .unwrap();

    let BuildRunOutcome::Complete {
        completed_steps, ..
    } = result.outcome()
    else {
        panic!("session orchestration should complete");
    };
    assert_eq!(completed_steps.len(), 2);
    assert_eq!(completed_steps[1].attempts_used(), 2);
    assert!(result.report().complete());
    assert_eq!(result.report().plan_id(), "session-demo-plan");
    let manifest = result.manifest().unwrap();
    assert_eq!(manifest.schema_version(), 1);
    assert_eq!(manifest.report(), result.report());
    let bundle = result.bundle().unwrap();
    assert_eq!(bundle.manifest(), manifest);
    assert_eq!(bundle.copied_artifacts().len(), 2);
    assert_eq!(
        std::fs::read_to_string(&manifest_path).unwrap(),
        manifest.to_json().unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/README.md")).unwrap(),
        "JSON factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
        "json-ready\n"
    );
    let transcript_service = provider.into_inner();
    assert_eq!(transcript_service.transcript().len(), 3);
    assert_eq!(transcript_service.transcript()[0].sequence_number(), 0);
    assert_eq!(
        transcript_service.transcript()[0]
            .request()
            .step()
            .step_id(),
        "readme"
    );
    assert!(transcript_service.transcript()[0].response_json().is_some());
    assert!(transcript_service.transcript()[0].error().is_none());
    assert_eq!(
        transcript_service.transcript()[1]
            .request()
            .step()
            .step_id(),
        "main"
    );
    assert!(transcript_service.transcript()[1]
        .request()
        .prior_failures()
        .is_empty());
    assert_eq!(
        transcript_service.transcript()[2]
            .request()
            .step()
            .step_id(),
        "main"
    );
    assert_eq!(
        transcript_service.transcript()[2]
            .request()
            .prior_failures()
            .len(),
        1
    );
    assert!(transcript_service.transcript()[2]
        .request_json()
        .contains("main-accept-1"));
    assert!(transcript_service.transcript()[2]
        .response_json()
        .unwrap()
        .contains("json-proposal-main-repaired"));
    let transcript_report = transcript_service.transcript_report().unwrap();
    let transcript_bundle_root = runtime.path().join("session/transcript-bundle");
    let transcript_bundle = journal
        .export_build_bundle_with_transcript(
            &loaded_plan,
            &transcript_bundle_root,
            &transcript_report,
        )
        .unwrap();
    let transcript_path = transcript_bundle_root.join("evidence/provider_transcript.json");
    assert_eq!(
        transcript_bundle.transcript_path(),
        Some(transcript_path.as_path())
    );
    assert_eq!(
        transcript_bundle.provider_transcript(),
        Some(&transcript_report)
    );
    let loaded_transcript = JsonProviderTranscript::read_json_file(&transcript_path).unwrap();
    assert_eq!(loaded_transcript, transcript_report);
    let verified_transcript_bundle = BuildBundle::read_verified(&transcript_bundle_root).unwrap();
    assert_eq!(
        verified_transcript_bundle.provider_transcript(),
        Some(&transcript_report)
    );
    assert_eq!(
        verified_transcript_bundle.transcript_path(),
        Some(transcript_path.as_path())
    );
    assert_eq!(loaded_transcript.entries().len(), 3);
    assert_eq!(
        loaded_transcript.entries()[2]
            .request()
            .prior_failures()
            .len(),
        1
    );

    let mut transcript_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&transcript_path).unwrap()).unwrap();
    let mut request_json_value: serde_json::Value = serde_json::from_str(
        transcript_value["entries"][2]["request_json"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    request_json_value["step"]["instruction"] =
        serde_json::Value::String("tampered but structurally valid".to_string());
    transcript_value["entries"][2]["request_json"] =
        serde_json::Value::String(request_json_value.to_string());
    std::fs::write(&transcript_path, transcript_value.to_string()).unwrap();
    assert!(BuildBundle::read_verified(&transcript_bundle_root)
        .unwrap_err()
        .contains("request JSON does not match decoded request"));
    assert_eq!(transcript_service.into_inner().requests.len(), 3);
}

#[test]
fn public_json_build_session_exports_transcript_bundle_in_one_call() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let plan = BuildPlan::new(
        "json-session-demo-plan",
        ExactRequest::new(
            "request-json-session-demo",
            "Build a JSON session demo with README and main artifacts.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::JSON factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::json-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-json-session-demo",
        "request-json-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let manifest_path = runtime.path().join("json-session/build-manifest.json");
    let bundle_root = runtime.path().join("json-session/bundle");
    let result = journal
        .run_json_build_session(
            &plan,
            JsonDemoService {
                requests: Vec::new(),
            },
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-json-session-build",
            &BuildSessionExportPaths::manifest_and_bundle(&manifest_path, &bundle_root),
        )
        .unwrap();

    assert!(result.session().report().complete());
    assert_eq!(result.provider_transcript().entries().len(), 3);
    assert_eq!(
        result.provider_transcript().entries()[2]
            .request()
            .prior_failures()
            .len(),
        1
    );
    assert_eq!(result.service().requests.len(), 3);
    let bundle = result.session().bundle().unwrap();
    assert!(bundle.provider_transcript().is_some());
    assert_eq!(
        bundle.provider_transcript().unwrap(),
        result.provider_transcript()
    );
    assert_eq!(
        std::fs::read_to_string(bundle.transcript_path().unwrap()).unwrap(),
        result.provider_transcript().to_json().unwrap()
    );
    let verified_bundle = BuildBundle::read_verified(&bundle_root).unwrap();
    assert_eq!(
        verified_bundle.provider_transcript().unwrap(),
        result.provider_transcript()
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
        "json-ready\n"
    );
}

#[test]
fn public_json_build_session_runs_from_frozen_plan_file_with_transcript_bundle() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let plan = BuildPlan::new(
        "json-file-session-demo-plan",
        ExactRequest::new(
            "request-json-file-session-demo",
            "Build a JSON file-backed session demo with README and main artifacts.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::JSON factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::json-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let plan_path = runtime.path().join("plans/json-session-plan.json");
    plan.write_json_file(&plan_path).unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-json-file-session-demo",
        "request-json-file-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let manifest_path = runtime.path().join("json-file-session/build-manifest.json");
    let bundle_root = runtime.path().join("json-file-session/bundle");

    let result = journal
        .run_json_build_session_from_plan_json_file(
            &plan_path,
            JsonDemoService {
                requests: Vec::new(),
            },
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-json-file-session-build",
            &BuildSessionExportPaths::manifest_and_bundle(&manifest_path, &bundle_root),
        )
        .unwrap();

    assert!(result.session().report().complete());
    assert_eq!(
        result.session().report().plan_id(),
        "json-file-session-demo-plan"
    );
    assert_eq!(result.provider_transcript().entries().len(), 3);
    assert_eq!(result.service().requests.len(), 3);
    assert_eq!(
        result.session().bundle().unwrap().provider_transcript(),
        Some(result.provider_transcript())
    );
    assert_eq!(
        BuildBundle::read_verified(&bundle_root)
            .unwrap()
            .provider_transcript()
            .unwrap(),
        result.provider_transcript()
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/README.md")).unwrap(),
        "JSON factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
        "json-ready\n"
    );
}

#[cfg(windows)]
#[test]
fn public_json_build_session_runs_with_external_command_provider() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let provider_script = runtime.path().join("provider.ps1");
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
    proposal_id = 'command-proposal-readme'
    summary = 'external command supplied README method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'README.md'; contents = "Command factory demo`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main' -and $failureCount -eq 0) {
  $proposal = [ordered]@{
    proposal_id = 'command-proposal-main-wrong-first'
    summary = 'external command supplied first main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "wrong`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main') {
  $proposal = [ordered]@{
    proposal_id = 'command-proposal-main-repaired'
    summary = 'external command supplied repaired main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "command-ready`n" } })
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
        "command-session-demo-plan",
        ExactRequest::new(
            "request-command-session-demo",
            "Build a command-backed session demo with README and main artifacts.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::Command factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::command-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let plan_path = runtime.path().join("plans/command-session-plan.json");
    plan.write_json_file(&plan_path).unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-command-session-demo",
        "request-command-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let provider_config = CommandJsonProviderConfig::new(
        "powershell",
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            provider_script.display().to_string(),
        ],
        None,
    );
    let provider_config_path = runtime.path().join("providers/command-provider.json");
    provider_config
        .write_json_file(&provider_config_path)
        .unwrap();
    let loaded_provider_config =
        CommandJsonProviderConfig::read_json_file(&provider_config_path).unwrap();
    assert_eq!(loaded_provider_config, provider_config);
    let provider = loaded_provider_config.to_service().unwrap();
    let bundle_root = runtime.path().join("command-session/bundle");

    let result = journal
        .run_json_build_session_from_plan_json_file(
            &plan_path,
            provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-command-session-build",
            &BuildSessionExportPaths::bundle(&bundle_root),
        )
        .unwrap();

    assert!(result.session().report().complete());
    assert_eq!(result.provider_transcript().entries().len(), 3);
    assert_eq!(
        result.provider_transcript().entries()[2]
            .request()
            .prior_failures()
            .len(),
        1
    );
    assert!(result.provider_transcript().entries()[2]
        .response_json()
        .unwrap()
        .contains("command-proposal-main-repaired"));
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "Command factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "command-ready\n"
    );
    let verified_bundle = BuildBundle::read_verified(&bundle_root).unwrap();
    assert_eq!(
        verified_bundle.provider_transcript().unwrap(),
        result.provider_transcript()
    );
}

#[cfg(windows)]
#[test]
fn public_command_json_build_session_runs_from_plan_and_provider_config_files() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let provider_script = runtime.path().join("configured-provider.ps1");
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
    proposal_id = 'configured-command-proposal-readme'
    summary = 'configured external command supplied README method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'README.md'; contents = "Configured command demo`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main' -and $failureCount -eq 0) {
  $proposal = [ordered]@{
    proposal_id = 'configured-command-proposal-main-wrong-first'
    summary = 'configured external command supplied first main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "wrong`n" } })
    suggested_verification = @()
  }
} elseif ($stepId -eq 'main') {
  $proposal = [ordered]@{
    proposal_id = 'configured-command-proposal-main-repaired'
    summary = 'configured external command supplied repaired main method'
    steps = @([ordered]@{ WriteFile = [ordered]@{ path = 'src/main.txt'; contents = "configured-ready`n" } })
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
        "configured-command-session-plan",
        ExactRequest::new(
            "request-configured-command-session",
            "Build a configured command-backed session demo.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::Configured command demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::configured-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let plan_path = runtime.path().join("plans/configured-command-plan.json");
    plan.write_json_file(&plan_path).unwrap();
    let provider_config_path = runtime
        .path()
        .join("providers/configured-command-provider.json");
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
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-configured-command-session",
        "request-configured-command-session",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let bundle_root = runtime.path().join("configured-command-session/bundle");

    let result = journal
        .run_command_json_build_session_from_files(
            &plan_path,
            &provider_config_path,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-configured-command-session",
            &BuildSessionExportPaths::bundle(&bundle_root),
        )
        .unwrap();

    assert!(result.session().report().complete());
    assert_eq!(result.provider_transcript().entries().len(), 3);
    assert!(result.provider_transcript().entries()[2]
        .response_json()
        .unwrap()
        .contains("configured-command-proposal-main-repaired"));
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("README.md")).unwrap(),
        "Configured command demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("src/main.txt")).unwrap(),
        "configured-ready\n"
    );
    let verified_bundle = BuildBundle::read_verified(&bundle_root).unwrap();
    assert_eq!(
        verified_bundle.provider_transcript().unwrap(),
        result.provider_transcript()
    );
}

#[cfg(windows)]
#[test]
fn external_command_provider_timeout_stops_without_execution() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let provider_script = runtime.path().join("slow-provider.ps1");
    std::fs::write(
        &provider_script,
        r#"
$ErrorActionPreference = 'Stop'
[Console]::In.ReadToEnd() | Out-Null
Start-Sleep -Seconds 5
"#,
    )
    .unwrap();
    let plan = BuildPlan::new(
        "timeout-session-demo-plan",
        ExactRequest::new(
            "request-timeout-session-demo",
            "Build a timeout demo README.",
        ),
        vec![BuildStepSpec::new(
            "readme",
            "Create the README artifact.",
            vec!["file_contains:README.md::should-not-exist".to_string()],
        )],
    )
    .unwrap();
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-timeout-session-demo",
        "request-timeout-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let provider_config = CommandJsonProviderConfig::new(
        "powershell",
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            provider_script.display().to_string(),
        ],
        Some(100),
    );
    let provider_config_path = runtime.path().join("providers/slow-provider.json");
    provider_config
        .write_json_file(&provider_config_path)
        .unwrap();
    let provider = CommandJsonProviderConfig::read_json_file(&provider_config_path)
        .unwrap()
        .to_service()
        .unwrap();
    assert_eq!(provider.timeout(), Some(Duration::from_millis(100)));

    let result = journal
        .run_json_build_session(
            &plan,
            provider,
            &BuildRunLimits::new(1).unwrap(),
            "operator-confirmed-timeout-session-build",
            &BuildSessionExportPaths::bundle(runtime.path().join("timeout-session/bundle")),
        )
        .unwrap();

    let BuildRunOutcome::Stopped {
        reason,
        journal_records,
        ..
    } = result.session().outcome()
    else {
        panic!("timed-out provider should stop the build");
    };
    assert!(reason.contains("proposal provider failed"));
    assert!(reason.contains("timed out"));
    assert!(result.session().bundle().is_none());
    assert!(!result.session().report().complete());
    assert_eq!(result.provider_transcript().entries().len(), 1);
    assert!(result.provider_transcript().entries()[0]
        .error()
        .unwrap()
        .contains("timed out"));
    assert!(result.provider_transcript().entries()[0]
        .response_json()
        .is_none());
    assert!(!journal_records
        .iter()
        .any(|record| record.record_kind() == RuntimeRecordKind::ExecutionReceipt));
    assert!(!workspace.path().join("README.md").exists());
}

#[test]
fn public_build_session_runs_from_frozen_plan_json_file() {
    let workspace = tempfile::tempdir().unwrap();
    let runtime = tempfile::tempdir().unwrap();
    let plan = BuildPlan::new(
        "file-session-demo-plan",
        ExactRequest::new(
            "request-file-session-demo",
            "Build a file-backed session demo with README and main artifacts.",
        ),
        vec![
            BuildStepSpec::new(
                "readme",
                "Create the README artifact.",
                vec!["file_contains:README.md::JSON factory demo".to_string()],
            ),
            BuildStepSpec::new(
                "main",
                "Create the main text artifact.",
                vec!["file_contains:src/main.txt::json-ready".to_string()],
            ),
        ],
    )
    .unwrap();
    let plan_path = runtime.path().join("plans/frozen-plan.json");
    plan.write_json_file(&plan_path).unwrap();
    assert_eq!(BuildPlan::read_json_file(&plan_path).unwrap(), plan);
    let journal = RuntimeJournal::new(
        runtime.path(),
        "trace-file-session-demo",
        "request-file-session-demo",
        HostBounds::new(workspace.path(), 4, 4096),
    );
    let mut provider = JsonBuildProposalProvider::new(JsonDemoService {
        requests: Vec::new(),
    });
    let manifest_path = runtime.path().join("file-session/build-manifest.json");
    let bundle_root = runtime.path().join("file-session/bundle");

    let result = journal
        .run_build_session_from_plan_json_file(
            &plan_path,
            &mut provider,
            &BuildRunLimits::new(2).unwrap(),
            "operator-confirmed-file-session-build",
            &BuildSessionExportPaths::manifest_and_bundle(&manifest_path, &bundle_root),
        )
        .unwrap();

    assert!(matches!(result.outcome(), BuildRunOutcome::Complete { .. }));
    assert!(result.report().complete());
    assert_eq!(result.report().plan_id(), "file-session-demo-plan");
    assert_eq!(
        std::fs::read_to_string(&manifest_path).unwrap(),
        result.manifest().unwrap().to_json().unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/README.md")).unwrap(),
        "JSON factory demo\n"
    );
    assert_eq!(
        std::fs::read_to_string(bundle_root.join("artifacts/src/main.txt")).unwrap(),
        "json-ready\n"
    );
    assert_eq!(result.bundle().unwrap().copied_artifacts().len(), 2);
    assert_eq!(provider.into_inner().requests.len(), 3);
}
