use chatty_factory_rebuild::{ExactRequest, RuntimeJournal};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let _ = journal.append_request(&ExactRequest::new("request", "text"));
    let _ = journal.append_confirmed_intent_receipt(String::new(), todo!());
    let _ = journal.append_proposal("attempt", String::new(), todo!());
    let _ = journal.append_gate_receipt(String::new(), todo!());
    let _ = journal.append_execution_receipt(String::new(), todo!());
    let _ = journal.append_vault_entry(String::new(), todo!());
    let _ = journal.append_triangulation_receipt(vec![], todo!());
    let _ = journal.append_promotion_candidate(String::new(), todo!());
    let _ = journal.append_promotion_approval(String::new(), todo!());
}
