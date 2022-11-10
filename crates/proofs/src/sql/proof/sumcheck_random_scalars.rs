use super::ProofCounts;

use curve25519_dalek::scalar::Scalar;

/// Accessor for the random scalars used to form the sumcheck polynomial of a query proof
pub struct SumcheckRandomScalars<'a> {
    pub entrywise_multipliers: &'a [Scalar],
    pub subpolynomial_multipliers: &'a [Scalar],
}

impl<'a> SumcheckRandomScalars<'a> {
    pub fn new(counts: &ProofCounts, scalars: &'a [Scalar]) -> Self {
        assert_eq!(scalars.len(), SumcheckRandomScalars::count(counts));
        let (entrywise_multipliers, subpolynomial_multipliers) =
            scalars.split_at(1 << counts.sumcheck_variables);
        Self {
            entrywise_multipliers,
            subpolynomial_multipliers,
        }
    }

    /// Count the number of random scalars required for sumcheck
    pub fn count(counts: &ProofCounts) -> usize {
        (1 << counts.sumcheck_variables) + counts.sumcheck_subpolynomials
    }
}
