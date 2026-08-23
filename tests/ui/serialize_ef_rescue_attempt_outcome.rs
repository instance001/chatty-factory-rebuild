use chatty_factory_rebuild::EfRescueAttemptOutcome;

fn assert_serialize<T: serde::Serialize>() {}

fn main() {
    assert_serialize::<EfRescueAttemptOutcome>();
}
