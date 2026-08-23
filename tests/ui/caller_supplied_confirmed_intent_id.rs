use chatty_factory_rebuild::{external_operator_assertion, RuntimeJournal};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let _ = journal.confirm_intent(
        "caller-picked-intent",
        todo!(),
        external_operator_assertion("assertion", "operator-confirmation"),
    );
}
