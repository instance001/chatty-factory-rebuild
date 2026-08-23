use chatty_factory_rebuild::{
    AllowedAttemptCapability, AuthorizedWorkOrderCapability, ConfirmedIntentCapability,
    JournalBackedFailureHandle, PromotedConstraint, PromotionCapability,
};

fn assert_serialize<T: serde::Serialize>() {}

fn main() {
    assert_serialize::<ConfirmedIntentCapability>();
    assert_serialize::<AllowedAttemptCapability>();
    assert_serialize::<AuthorizedWorkOrderCapability>();
    assert_serialize::<JournalBackedFailureHandle>();
    assert_serialize::<PromotionCapability>();
    assert_serialize::<PromotedConstraint>();
}
