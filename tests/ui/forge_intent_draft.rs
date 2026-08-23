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
    let mut draft = IntentDraft::new("draft", ExactRequest::new("request", "hello"), vec![claim]);
    draft.derived_claims = vec![];
}
