use chatty_factory_rebuild::RuntimeJournal;

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request");
    let _ = journal.issue_allowed_attempt("caller-picked-attempt", todo!(), todo!(), todo!(), &[]);
    let _ = journal.run_ef_rescue_attempt("caller-picked-attempt", todo!(), todo!(), todo!(), &[]);
}
