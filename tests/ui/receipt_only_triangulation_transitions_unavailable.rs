use chatty_factory_rebuild::{
    mark_triangulation_dormant, resume_triangulation, success_does_not_resolve_prior_evidence,
};

fn main() {
    let _ = mark_triangulation_dormant(todo!(), "caller-controlled dormant state");
    let _ = resume_triangulation(todo!(), todo!(), todo!());
    let _ = success_does_not_resolve_prior_evidence(todo!(), "caller-controlled success state");
}
