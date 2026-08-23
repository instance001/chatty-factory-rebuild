use chatty_factory_rebuild::{
    ExactRequest, IntentClaim, IntentClaimKind, IntentDraft, SourceSpan,
};

fn main() {
    let claim = IntentClaim::new(
        "claim",
        IntentClaimKind::AcceptanceCriterion,
        "file_contains:README.md::hello",
        vec![SourceSpan::new(0, 5, "hello")],
    );
    let _ = IntentDraft {
        draft_id: "draft".to_string(),
        exact_request: ExactRequest::new("request", "hello"),
        derived_claims: vec![claim],
    };
}
