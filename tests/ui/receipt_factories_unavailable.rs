use chatty_factory_rebuild::{gate_attempt_receipt, verify_against_intent};

fn main() {
    let _gate = gate_attempt_receipt("trace", "attempt", todo!(), todo!(), todo!(), &[]);
    let _verification = verify_against_intent(todo!(), todo!(), todo!());
}
