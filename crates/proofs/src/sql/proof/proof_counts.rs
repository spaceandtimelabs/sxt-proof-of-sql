use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Counters for different terms used within a proof
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProofCounts {
    /// TODO: add docs
    pub sumcheck_max_multiplicands: usize,
    /// TODO: add docs
    pub result_columns: usize,
    /// TODO: add docs
    pub anchored_mles: usize,
    /// TODO: add docs
    pub intermediate_mles: usize,
    /// TODO: add docs
    pub sumcheck_subpolynomials: usize,

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub post_result_challenges: usize,
}

impl ProofCounts {
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_acounts.annotate_trace",
        level = "info",
        skip_all
    )]
    /// TODO: add docs
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
