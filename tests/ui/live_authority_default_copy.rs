use chatty_factory_rebuild::{
    AllowedAttemptCapability, AuthorizedWorkOrderCapability, ConfirmedIntentCapability,
    JournalBackedFailureHandle, PromotedConstraint, PromotionCapability,
};

fn assert_default<T: Default>() {}
fn assert_copy<T: Copy>() {}

fn main() {
    assert_default::<ConfirmedIntentCapability>();
    assert_default::<AllowedAttemptCapability>();
    assert_default::<AuthorizedWorkOrderCapability>();
    assert_default::<JournalBackedFailureHandle>();
    assert_default::<PromotionCapability>();
    assert_default::<PromotedConstraint>();

    assert_copy::<ConfirmedIntentCapability>();
    assert_copy::<AllowedAttemptCapability>();
    assert_copy::<AuthorizedWorkOrderCapability>();
    assert_copy::<JournalBackedFailureHandle>();
    assert_copy::<PromotionCapability>();
    assert_copy::<PromotedConstraint>();
}
