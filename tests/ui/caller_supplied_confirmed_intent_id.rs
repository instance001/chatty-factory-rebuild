use chatty_factory_rebuild::{confirm_intent, external_operator_assertion};

fn main() {
    let _ = confirm_intent(
        "caller-picked-intent",
        todo!(),
        external_operator_assertion("assertion", "operator-confirmation"),
    );
}
