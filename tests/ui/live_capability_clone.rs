use chatty_factory_rebuild::{
    AllowedAttemptCapability, AuthorizedWorkOrderCapability, ConfirmedIntentCapability,
    JournalBackedFailureHandle, PromotionCapability,
};

fn assert_clone<T: Clone>() {}

fn main() {
    assert_clone::<ConfirmedIntentCapability>();
    assert_clone::<AllowedAttemptCapability>();
    assert_clone::<AuthorizedWorkOrderCapability>();
    assert_clone::<JournalBackedFailureHandle>();
    assert_clone::<PromotionCapability>();
}
