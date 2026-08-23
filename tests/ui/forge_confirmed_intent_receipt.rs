use chatty_factory_rebuild::{
    external_operator_assertion, ExactRequest, IntentClaim, IntentClaimKind, IntentDraft,
    RuntimeJournal, SourceSpan,
};

fn main() {
    let request = ExactRequest::new("request", "hello");
    let claim = IntentClaim::new(
        "claim",
        IntentClaimKind::AcceptanceCriterion,
        "file_contains:README.md::hello",
        vec![SourceSpan::new(0, 5, "hello")],
    );
    let draft = IntentDraft::new("draft", request, vec![claim]);
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let confirmed = journal
        .confirm_intent(draft, external_operator_assertion("operator"))
        .unwrap();
    let mut receipt = confirmed.receipt().clone();
    receipt.receipt_id = "caller-picked".to_string();
}

