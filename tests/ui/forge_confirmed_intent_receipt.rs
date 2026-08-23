use chatty_factory_rebuild::{
    confirm_intent, external_operator_assertion, ExactRequest, IntentClaim, IntentClaimKind,
    IntentDraft, SourceSpan,
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
    let confirmed = confirm_intent(draft, external_operator_assertion("operator")).unwrap();
    let mut receipt = confirmed.receipt().clone();
    receipt.receipt_id = "caller-picked".to_string();
}
