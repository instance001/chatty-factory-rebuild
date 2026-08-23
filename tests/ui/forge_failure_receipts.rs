use chatty_factory_rebuild::{
    CapabilitySpendReceipt, ConstraintPromotionCandidateReceipt, FailureEvidenceReceipt,
    FailureObservationReceipt, PromotionApprovalReceipt, RuntimeJournal, TriangulationReceipt,
    VaultEntryReceipt,
};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let records = journal.verify().unwrap();

    let mut failure: FailureEvidenceReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    failure.evidence = vec!["caller rewrote failure evidence".to_string()];

    let mut vault: VaultEntryReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    vault.lock_signals = vec!["caller rewrote vault lock".to_string()];

    let mut observation: FailureObservationReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    observation.scope = "caller/wide".to_string();

    let mut triangulation: TriangulationReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    triangulation.reason = Some("caller resolved it".to_string());

    let mut candidate: ConstraintPromotionCandidateReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    candidate.scope = "caller/global".to_string();

    let mut approval: PromotionApprovalReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    approval.candidate_receipt_hash = "caller-approved".to_string();

    let mut spend: CapabilitySpendReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    spend.consumed_for = "caller-replay".to_string();
}
