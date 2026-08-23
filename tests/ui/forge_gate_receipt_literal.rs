use chatty_factory_rebuild::GateReceipt;

fn main() {
    let _ = GateReceipt {
        receipt_id: "gate-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        request_hash: "request-hash".to_string(),
        confirmed_intent_receipt_id: "confirmed".to_string(),
        confirmed_intent_receipt_hash: "confirmed-hash".to_string(),
        attempt_id: "attempt-1".to_string(),
        proposal_id: "proposal".to_string(),
        proposal_hash: "proposal-hash".to_string(),
        admissible: true,
        reasons: vec![],
        blocked_by_constraint_ids: vec![],
    };
}
