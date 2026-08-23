use chatty_factory_rebuild::SourceSpan;

fn main() {
    let mut span = SourceSpan::new(0, 4, "text");
    span.exact_text = "changed".to_string();
}
