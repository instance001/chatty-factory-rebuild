use chatty_factory_rebuild::{AuthorizedWorkOrderCapability, HostBounds, RuntimeJournal};

fn execute_twice(
    journal: &RuntimeJournal,
    work_order: AuthorizedWorkOrderCapability,
    bounds: &HostBounds,
) {
    let _ = journal.execute_work_order(work_order, bounds);
    let _ = journal.execute_work_order(work_order, bounds);
}

fn main() {}
