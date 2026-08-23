use chatty_factory_rebuild::HostBounds;
use std::path::PathBuf;

fn main() {
    let _ = HostBounds {
        workspace_root: PathBuf::from("."),
        max_steps: 4,
        max_file_bytes: 4096,
    };
}
