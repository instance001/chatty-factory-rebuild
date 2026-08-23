use chatty_factory_rebuild::ExactRequest;

fn main() {
    let _ = ExactRequest {
        request_id: "request".to_string(),
        text: "original".to_string(),
        bytes_sha256: "caller-picked-hash".to_string(),
        byte_len: 999,
    };
}
