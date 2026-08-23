use chatty_factory_rebuild::EfRescueAttemptOutcome;

fn main() {
    let _ = EfRescueAttemptOutcome::Artifact {
        execution_record_id: "execution-record".to_string(),
        verification_record_id: "verification-record".to_string(),
        execution: todo!(),
        verification: todo!(),
    };

    let _ = EfRescueAttemptOutcome::UnresolvedFailure {
        failure_class: todo!(),
        gate_record_id: "gate-record".to_string(),
        failure_record_id: "failure-record".to_string(),
        vault_record_id: "vault-record".to_string(),
        observation_record_id: "observation-record".to_string(),
        gate: todo!(),
        failure: todo!(),
        vault: todo!(),
        observation: todo!(),
    };
}
