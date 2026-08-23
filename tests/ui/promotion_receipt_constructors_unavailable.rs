use chatty_factory_rebuild::{approve_promotion_receipt, candidate_from_triangulation};

fn main() {
    let _candidate = candidate_from_triangulation("candidate", todo!(), "tri-hash");
    let _approval = approve_promotion_receipt("approval", todo!(), "candidate-hash", todo!());
}
