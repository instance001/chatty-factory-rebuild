use chatty_factory_rebuild::{
    external_operator_assertion, ExactRequest, IntentClaim, IntentClaimKind, IntentDraft,
    MethodProposal, ProposedStep, RuntimeJournal, SourceSpan,
};
use std::path::PathBuf;

fn main() {
    let request = ExactRequest::new("request", "hello");
    let claim = IntentClaim::new(
        "claim",
        IntentClaimKind::AcceptanceCriterion,
        "file_contains:README.md::hello",
        vec![SourceSpan::new(0, 5, "hello")],
    );
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let intent = journal
        .confirm_intent(
            IntentDraft::new("draft", request, vec![claim]),
            external_operator_assertion("operator"),
        )
        .unwrap();
    let proposal = MethodProposal::new(
        "proposal",
        "external method",
        vec![ProposedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello".to_string(),
        }],
        vec![],
    );
    let allowed = journal
        .issue_allowed_attempt(&intent, &proposal, &[])
        .unwrap();
    let work_order = journal
        .authorize_work_order(allowed, &intent, &proposal)
        .unwrap();
    let mut receipt = work_order.receipt().clone();
    receipt.steps = vec![];
}

