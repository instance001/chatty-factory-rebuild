use chatty_factory_rebuild::{BoundedStep, WorkOrderReceipt};
use std::path::PathBuf;

fn main() {
    let _ = WorkOrderReceipt {
        receipt_id: "work-order-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        request_hash: "request-hash".to_string(),
        confirmed_intent_receipt_id: "confirmed".to_string(),
        confirmed_intent_receipt_hash: "confirmed-hash".to_string(),
        attempt_id: "attempt-1".to_string(),
        proposal_id: "proposal".to_string(),
        proposal_hash: "proposal-hash".to_string(),
        gate_receipt_id: "gate-attempt-1".to_string(),
        gate_receipt_hash: "gate-hash".to_string(),
        steps: vec![BoundedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello".to_string(),
        }],
    };
}
