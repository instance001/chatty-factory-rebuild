use chatty_factory_rebuild::AuthorizedWorkOrderCapability;

fn main() {
    let _forged = AuthorizedWorkOrderCapability {
        receipt: panic!("receipt unavailable"),
        receipt_hash: "hash".to_string(),
    };
}
