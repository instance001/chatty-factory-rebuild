use chatty_factory_rebuild::{IntentClaim, IntentClaimKind, SourceSpan};

fn main() {
    let mut claim = IntentClaim::new(
        "claim",
        IntentClaimKind::AcceptanceCriterion,
        "file_contains:README.md::hello",
        vec![SourceSpan::new(0, 5, "hello")],
    );
    claim.text = "changed".to_string();
}
