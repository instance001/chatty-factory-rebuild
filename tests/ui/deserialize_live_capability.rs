use chatty_factory_rebuild::ConfirmedIntentCapability;

fn main() {
    let _capability: ConfirmedIntentCapability = serde_json::from_str("{}").unwrap();
}
