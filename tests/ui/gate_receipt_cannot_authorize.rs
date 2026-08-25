use chatty_factory_rebuild::{GateReceipt, HostBounds, RuntimeJournal};

fn main() {
    let journal = RuntimeJournal::new(".", "trace-1", "request-1", HostBounds::new(".", 4, 4096));
    let gate: GateReceipt = serde_json::from_str("{}").unwrap();

    let _ = journal.authorize_work_order(gate, todo!(), todo!());
}
