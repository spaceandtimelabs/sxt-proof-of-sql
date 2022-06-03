mod message;
pub use message::{ProverMessage, VerifierMsg};

mod proof;
#[cfg(test)]
mod proof_test;
pub use proof::SumcheckProof;

mod prover_state;
pub use prover_state::{init_prover_state, ProverState};
