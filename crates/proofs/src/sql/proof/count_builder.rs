use crate::sql::proof::ProofCounts;

use crate::base::proof::ProofError;
use std::cmp::max;

/// State used to count the number of components in a proof
pub struct CountBuilder {
    counts: ProofCounts,
}

impl CountBuilder {
    pub fn new() -> Self {
        Self {
            counts: ProofCounts::default(),
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
        // Note: in future PRs, this will be able to error
        // if state is not valid.
        Ok(self.counts)
    }
}

impl Default for CountBuilder {
    fn default() -> Self {
        Self::new()
    }
}
