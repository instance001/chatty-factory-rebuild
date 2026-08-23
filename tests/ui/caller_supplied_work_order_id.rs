use chatty_factory_rebuild::RuntimeJournal;

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let _ = journal.authorize_work_order("caller-picked-work-order", todo!(), todo!(), todo!());
}
