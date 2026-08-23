use chatty_factory_rebuild::{
    AllowedAttemptCapability, AuthorizedWorkOrderCapability, ConfirmedIntentCapability,
    JournalBackedFailureHandle, PromotedConstraint, PromotionCapability,
};

fn main() {
    let _confirmed: ConfirmedIntentCapability = serde_json::from_str("{}").unwrap();
    let _allowed: AllowedAttemptCapability = serde_json::from_str("{}").unwrap();
    let _work_order: AuthorizedWorkOrderCapability = serde_json::from_str("{}").unwrap();
    let _failure_handle: JournalBackedFailureHandle = serde_json::from_str("{}").unwrap();
    let _promotion: PromotionCapability = serde_json::from_str("{}").unwrap();
    let _constraint: PromotedConstraint = serde_json::from_str("{}").unwrap();
}
