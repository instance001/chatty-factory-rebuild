use chatty_factory_rebuild::SourceSpan;

fn main() {
    let _ = SourceSpan {
        start: 0,
        end: 4,
        exact_text: "caller-picked".to_string(),
    };
}
