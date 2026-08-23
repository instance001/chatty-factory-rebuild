use chatty_factory_rebuild::{RuntimeJournal, RuntimeRecordKind};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let _ = journal.append_record(
        RuntimeRecordKind::TriangulationReceipt,
        None,
        vec![],
        &serde_json::json!({"forged": true}),
    );
}

