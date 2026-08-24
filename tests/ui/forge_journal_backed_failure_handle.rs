use chatty_factory_rebuild::JournalBackedFailureHandle;

fn main() {
    let _handle = JournalBackedFailureHandle {
        vault_record_id: "vault-record".to_string(),
        vault_record_hash: "vault-hash".to_string(),
        receipt: todo!(),
    };
}
