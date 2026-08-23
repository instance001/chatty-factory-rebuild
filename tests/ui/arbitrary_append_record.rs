use chatty_factory_rebuild::{RuntimeJournal, RuntimeRecordKind};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let _ = journal.append_record(
        RuntimeRecordKind::TriangulationReceipt,
        None,
        vec![],
        &serde_json::json!({"forged": true}),
    );
}
