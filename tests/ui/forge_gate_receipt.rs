use chatty_factory_rebuild::{MethodProposal, ProposedStep, RuntimeJournal};
use std::path::PathBuf;

fn main() {
    let journal = RuntimeJournal::new(".", "trace", "request", chatty_factory_rebuild::HostBounds::new(".", 4, 4096));
    let proposal = MethodProposal::new(
        "proposal",
        "external method",
        vec![ProposedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello".to_string(),
        }],
        vec![],
    );
    let mut gate = journal.issue_allowed_attempt(todo!(), &proposal, &[]).unwrap_err();
    gate.admissible = true;
}

