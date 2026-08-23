use chatty_factory_rebuild::{IntentClaim, IntentClaimKind, SourceSpan};

fn main() {
    let _ = IntentClaim {
        claim_id: "claim".to_string(),
        kind: IntentClaimKind::AcceptanceCriterion,
        text: "file_contains:README.md::hello".to_string(),
        source_spans: vec![SourceSpan::new(0, 5, "hello")],
    };
}
