use chatty_factory_rebuild::{MethodProposal, ProposedStep};
use std::path::PathBuf;

fn main() {
    let _ = MethodProposal {
        proposal_id: "proposal".to_string(),
        summary: "external method".to_string(),
        steps: vec![ProposedStep::WriteFile {
            path: PathBuf::from("README.md"),
            contents: "hello".to_string(),
        }],
        suggested_verification: vec!["model-suggested verification".to_string()],
    };
}
