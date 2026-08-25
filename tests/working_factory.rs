use std::path::PathBuf;

use chatty_factory_rebuild::{
    external_operator_assertion, BuildFailureContext, BuildPlan, BuildProposalContext,
    BuildProposalProvider, BuildRunLimits, BuildRunOutcome, BuildStepSpec, EfRescueAttemptOutcome,
    ExactRequest, HostBounds, IntentClaim, IntentClaimKind, IntentDraft, MethodProposal,
    ProposedStep, RuntimeJournal, RuntimeRecordKind, SourceSpan,
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
