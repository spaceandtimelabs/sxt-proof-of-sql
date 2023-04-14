use super::{compute_evaluation_vector, ProofCounts};

use curve25519_dalek::scalar::Scalar;

/// Accessor for the random scalars used to form the sumcheck polynomial of a query proof
pub struct SumcheckRandomScalars<'a> {
    entrywise_point: &'a [Scalar],
    pub subpolynomial_multipliers: &'a [Scalar],
    table_length: usize,
}

impl<'a> SumcheckRandomScalars<'a> {
    pub fn new(counts: &ProofCounts, scalars: &'a [Scalar]) -> Self {
        assert_eq!(scalars.len(), SumcheckRandomScalars::count(counts));
        let (entrywise_point, subpolynomial_multipliers) =
            scalars.split_at(counts.sumcheck_variables);
        Self {
            entrywise_point,
            subpolynomial_multipliers,
            table_length: counts.table_length,
        }
    }

    pub fn compute_entrywise_multipliers(&self) -> Vec<Scalar> {
        let mut v = vec![Scalar::zero(); self.table_length];
        compute_evaluation_vector(&mut v, self.entrywise_point);
        v
    }

    /// Count the number of random scalars required for sumcheck
    pub fn count(counts: &ProofCounts) -> usize {
        counts.sumcheck_variables + counts.sumcheck_subpolynomials
    }
}
