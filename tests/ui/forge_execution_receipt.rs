use chatty_factory_rebuild::{ExecutionReceipt, RuntimeJournal};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let records = journal.verify().unwrap();
    let mut execution: ExecutionReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    execution.executed_steps = 99;
}

