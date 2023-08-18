use crate::sql::proof::ProofCounts;

use crate::base::bit::BitDistribution;
use crate::base::proof::ProofError;
use std::cmp::max;

/// Track the number of components expected for in a query's proof
pub struct CountBuilder<'a> {
    bit_distributions: &'a [BitDistribution],
    counts: ProofCounts,
}

impl<'a> CountBuilder<'a> {
    pub fn new(bit_distributions: &'a [BitDistribution]) -> Self {
        Self {
            bit_distributions,
            counts: Default::default(),
        }
    }

    /// Proof counts can be dependent on how bits are distributed in a column of data.
    ///
    /// This method provides access to the bit distributions of a proof during the counting
    /// pass of verification.
    pub fn consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofError> {
        if self.bit_distributions.is_empty() {
            Err(ProofError::VerificationError(
                "expected prover to provide bit distribution",
            ))
        } else {
            let res = self.bit_distributions[0].clone();
            self.bit_distributions = &self.bit_distributions[1..];
            Ok(res)
        }
    }

    pub fn count_result_columns(&mut self, cnt: usize) {
        self.counts.result_columns += cnt;
    }

    pub fn count_subpolynomials(&mut self, cnt: usize) {
        self.counts.sumcheck_subpolynomials += cnt;
    }

    pub fn count_anchored_mles(&mut self, cnt: usize) {
        self.counts.anchored_mles += cnt;
    }

    pub fn count_intermediate_mles(&mut self, cnt: usize) {
        self.counts.intermediate_mles += cnt;
    }

    pub fn count_degree(&mut self, degree: usize) {
        self.counts.sumcheck_max_multiplicands =
            max(self.counts.sumcheck_max_multiplicands, degree);
    }

    pub fn counts(&self) -> Result<ProofCounts, ProofError> {
        if !self.bit_distributions.is_empty() {
            return Err(ProofError::VerificationError(
                "incorrect number of bit distributions provided",
            ));
        }
        Ok(self.counts)
    }
}
