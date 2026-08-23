use chatty_factory_rebuild::{HostBounds, RuntimeJournal};

fn main() {
    let bounds = HostBounds::new(".", 4, 4096);
    let journal = RuntimeJournal::new(".", "trace", "request", bounds.clone());

    let _ = journal.issue_allowed_attempt(todo!(), todo!(), &bounds, &[]);
    let _ = journal.execute_work_order(todo!(), &bounds);
    let _ = journal.run_ef_rescue_attempt(todo!(), todo!(), &bounds, &[]);
}
