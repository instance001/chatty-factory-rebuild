use chatty_factory_rebuild::{HostBounds, PromotedConstraint, RuntimeJournal};

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", HostBounds::new(".", 4, 4096));
    let constraints: Vec<PromotedConstraint> = Vec::new();

    let _ = journal.issue_allowed_attempt(todo!(), todo!(), &constraints);
    let _ = journal.run_ef_rescue_attempt(todo!(), todo!(), &constraints);
}
