use chatty_factory_rebuild::RuntimeJournal;

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let _ = journal.issue_allowed_attempt("caller-picked-attempt", todo!(), todo!(), todo!(), &[]);
    let _ = journal.run_ef_rescue_attempt("caller-picked-attempt", todo!(), todo!(), todo!(), &[]);
}

