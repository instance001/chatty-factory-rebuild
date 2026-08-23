use chatty_factory_rebuild::{PromotedConstraint, PromotionCapability};

fn main() {
    let _forged_capability = PromotionCapability {
        capability_id: "cap-promotion-forged".to_string(),
        approval_record_id: "approval-record".to_string(),
        approval_record_hash: "approval-record-hash".to_string(),
        approval_receipt: panic!("approval unavailable"),
        approval_receipt_hash: "hash".to_string(),
        candidate_receipt: panic!("candidate unavailable"),
    };
    let _forged_constraint = PromotedConstraint {
        constraint_id: "constraint-forged".to_string(),
        trace_id: "trace-1".to_string(),
        request_id: "request-1".to_string(),
        scope: "write:README.md".to_string(),
        lock_signal: "write:README.md".to_string(),
        promotion_approval_receipt_id: "approval-forged".to_string(),
        promotion_approval_receipt_hash: "hash".to_string(),
    };
}
