use chatty_factory_rebuild::ExternalOperatorAssertionReceipt;

fn main() {
    let _ = ExternalOperatorAssertionReceipt {
        assertion_id: "caller-picked".to_string(),
        asserted_context: "operator-confirmation".to_string(),
        statement: "cryptographically verified human identity".to_string(),
    };
}
