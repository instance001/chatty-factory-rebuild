use chatty_factory_rebuild::{
    external_operator_assertion, ConfirmedIntentReceipt, ExactRequest, IntentClaim,
    IntentClaimKind, SourceSpan,
};

fn main() {
    let request = ExactRequest::new("request", "hello");
    let claim = IntentClaim::new(
        "claim",
        IntentClaimKind::AcceptanceCriterion,
        "file_contains:README.md::hello",
        vec![SourceSpan::new(0, 5, "hello")],
    );
    let _ = ConfirmedIntentReceipt {
        receipt_id: "caller-picked".to_string(),
        exact_request: request,
        derived_claims: vec![claim],
        confirmation_assertion: external_operator_assertion("operator"),
    };
}
