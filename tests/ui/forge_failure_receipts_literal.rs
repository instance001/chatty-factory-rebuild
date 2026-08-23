use chatty_factory_rebuild::{
    external_operator_assertion, CapabilitySpendReceipt, ConstraintPromotionCandidateReceipt,
    FailureClass, FailureEvidenceReceipt, FailureObservationReceipt, PromotionApprovalReceipt,
    TriangulationReceipt, TriangulationStatus, VaultEntryReceipt,
};

fn main() {
    let _failure = FailureEvidenceReceipt {
        receipt_id: "failure-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: "attempt-1".to_string(),
        parent_receipt_id: "verification-attempt-1".to_string(),
        parent_receipt_hash: "verification-hash".to_string(),
        failure_class: FailureClass::VerificationFailed,
        evidence: vec!["model said no".to_string()],
        lock_signals: vec!["same-label".to_string()],
    };

    let _vault = VaultEntryReceipt {
        receipt_id: "vault-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: "attempt-1".to_string(),
        failure_evidence_receipt_id: "failure-attempt-1".to_string(),
        failure_evidence_receipt_hash: "failure-hash".to_string(),
        failure_class: FailureClass::VerificationFailed,
        evidence: vec!["model said no".to_string()],
        lock_signals: vec!["same-label".to_string()],
    };

    let _observation = FailureObservationReceipt {
        receipt_id: "observation-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: "attempt-1".to_string(),
        vault_entry_receipt_id: "vault-attempt-1".to_string(),
        vault_entry_receipt_hash: "vault-hash".to_string(),
        scope: "path".to_string(),
        lock_signal: "same-label".to_string(),
        evidence: vec!["model said no".to_string()],
    };

    let _triangulation = TriangulationReceipt {
        receipt_id: "triangulation-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        source_vault_record_ids: vec!["vault-record-1".to_string(), "vault-record-2".to_string()],
        source_vault_record_hashes: vec!["hash-1".to_string(), "hash-2".to_string()],
        source_attempt_ids: vec!["attempt-1".to_string(), "attempt-2".to_string()],
        status: TriangulationStatus::Isolated,
        lock_signal: Some("same-label".to_string()),
        isolated_fault_condition: Some("specific bounded condition".to_string()),
        reason: Some("caller made a law".to_string()),
    };

    let _candidate = ConstraintPromotionCandidateReceipt {
        receipt_id: "candidate-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        triangulation_receipt_id: "triangulation-1".to_string(),
        triangulation_receipt_hash: "triangulation-hash".to_string(),
        source_vault_record_ids: vec!["vault-record-1".to_string(), "vault-record-2".to_string()],
        source_vault_record_hashes: vec!["hash-1".to_string(), "hash-2".to_string()],
        scope: "global".to_string(),
        lock_signal: "same-label".to_string(),
        isolated_fault_condition: "specific bounded condition".to_string(),
    };

    let _approval = PromotionApprovalReceipt {
        receipt_id: "approval-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        candidate_receipt_id: "candidate-1".to_string(),
        candidate_receipt_hash: "candidate-hash".to_string(),
        approval_assertion: external_operator_assertion("promotion"),
    };

    let _spend = CapabilitySpendReceipt {
        receipt_id: "spend-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        capability_id: "capability-1".to_string(),
        consumed_for: "execution".to_string(),
        consumed_receipt_id: "receipt-1".to_string(),
        consumed_receipt_hash: "receipt-hash".to_string(),
        consumed_record_id: "record-1".to_string(),
        consumed_record_hash: "record-hash".to_string(),
    };
}
