use chatty_factory_rebuild::VerificationReceipt;

fn main() {
    let _ = VerificationReceipt {
        receipt_id: "verification-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: "attempt-1".to_string(),
        execution_receipt_id: "execution-attempt-1".to_string(),
        execution_receipt_hash: "execution-hash".to_string(),
        success: true,
        checked_claim_ids: vec!["claim".to_string()],
        evidence: vec!["model said it passed".to_string()],
    };
}
