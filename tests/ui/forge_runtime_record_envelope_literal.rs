use chatty_factory_rebuild::{RuntimeRecordEnvelope, RuntimeRecordKind};

fn main() {
    let _ = RuntimeRecordEnvelope {
        record_id: "triangulation-0".to_string(),
        sequence_number: 0,
        record_kind: RuntimeRecordKind::TriangulationReceipt,
        trace_id: "trace".to_string(),
        request_id: "request".to_string(),
        attempt_id: None,
        parent_record_ids: vec![],
        previous_record_hash: None,
        payload_hash: "payload-hash".to_string(),
        payload: serde_json::json!({"forged": true}),
        record_hash: "record-hash".to_string(),
    };
}
