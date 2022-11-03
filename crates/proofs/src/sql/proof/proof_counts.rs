use std::fmt::Debug;

/// Counters for different terms used within a proof
#[derive(Default, Debug, Clone, Copy)]
pub struct ProofCounts {
    pub sumcheck_variables: usize,
    pub result_columns: usize,
    pub anchored_mles: usize,
    pub intermediate_mles: usize,
    pub sumcheck_subpolynomials: usize,
}
