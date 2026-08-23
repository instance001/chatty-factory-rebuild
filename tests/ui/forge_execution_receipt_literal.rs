use chatty_factory_rebuild::ExecutionReceipt;
use std::path::PathBuf;

fn main() {
    let _ = ExecutionReceipt {
        receipt_id: "execution-attempt-1".to_string(),
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: "attempt-1".to_string(),
        work_order_receipt_id: "work-order-attempt-1".to_string(),
        work_order_receipt_hash: "work-order-hash".to_string(),
        executed_steps: 1,
        written_files: vec![PathBuf::from("README.md")],
    };
}
