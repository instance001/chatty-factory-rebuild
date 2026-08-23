use chatty_factory_rebuild::{RuntimeJournal, VerificationReceipt};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let records = journal.verify().unwrap();
    let mut verification: VerificationReceipt =
        serde_json::from_value(records[0].payload().clone()).unwrap();
    verification.success = true;
}

