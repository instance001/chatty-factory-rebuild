use chatty_factory_rebuild::HostBounds;

fn main() {
    let mut bounds = HostBounds::new(".", 4, 4096);
    bounds.max_steps = 99;
}
