use chatty_factory_rebuild::ExactRequest;

fn main() {
    let mut request = ExactRequest {
        request_id: "request".to_string(),
        text: "original".to_string(),
        bytes_sha256: "caller-picked-hash".to_string(),
        byte_len: 999,
    };
    request.bytes_sha256 = "modified-hash".to_string();
}
