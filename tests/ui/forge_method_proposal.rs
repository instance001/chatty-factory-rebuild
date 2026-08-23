use chatty_factory_rebuild::{MethodProposal, ProposedStep};
use std::path::PathBuf;

fn main() {
    let mut proposal = MethodProposal::new(
        "proposal",
        "external method",
        vec![ProposedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello".to_string(),
        }],
        vec!["model-suggested verification".to_string()],
    );
    proposal.steps = vec![];
}
