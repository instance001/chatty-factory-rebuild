use chatty_factory_rebuild::RuntimeJournal;

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let _ = journal.authorize_work_order("caller-picked-work-order", todo!(), todo!(), todo!());
}

