use core::fmt::Debug;
use serde::{Deserialize, Serialize};

/// Counters for different terms used within a proof
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProofCounts {
    pub sumcheck_max_multiplicands: usize,
    pub result_columns: usize,
    pub anchored_mles: usize,
    pub intermediate_mles: usize,
    pub sumcheck_subpolynomials: usize,

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub post_result_challenges: usize,
}

impl ProofCounts {
    #[tracing::instrument(name = "ProofCounts::annotate_trace", level = "debug", skip_all)]
    pub fn annotate_trace(&self) {
        tracing::info!(
            "sumcheck_max_multiplicands = {:?}",
            self.sumcheck_max_multiplicands
        );
        tracing::info!("result_columns = {:?}", self.result_columns);
        tracing::info!("anchored_mles = {:?}", self.anchored_mles);
        tracing::info!("intermediate_mles = {:?}", self.intermediate_mles);
        tracing::info!(
            "sumcheck_subpolynomials = {:?}",
            self.sumcheck_subpolynomials
        );
        tracing::info!("post_result_challenges = {:?}", self.post_result_challenges);
    }
}
