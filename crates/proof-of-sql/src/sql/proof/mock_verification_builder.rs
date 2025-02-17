use super::{SumcheckSubpolynomialType, VerificationBuilder};
use crate::base::{bit::BitDistribution, proof::ProofSizeMismatch, scalar::Scalar};
use alloc::vec::Vec;
use core::iter;

/// Track components used to verify a query's proof
pub struct MockVerificationBuilder<S: Scalar> {
    bit_distributions: Vec<BitDistribution>,
    bit_distribution_offset: usize,
    consumed_final_round_pcs_proof_mles: usize,
    subpolynomial_max_multiplicands: usize,

    evaluation_row_index: usize,
    final_round_mles: Vec<Vec<S>>,
    pub(crate) identity_subpolynomial_evaluations: Vec<Vec<S>>,
    pub(crate) zerosum_subpolynomial_evaluations: Vec<Vec<S>>,
}

impl<S: Scalar> VerificationBuilder<S> for MockVerificationBuilder<S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        match subpolynomial_type {
            SumcheckSubpolynomialType::Identity => {
                if self.identity_subpolynomial_evaluations.len() <= self.evaluation_row_index {
                    self.identity_subpolynomial_evaluations
                        .extend(iter::repeat(Vec::new()).take(
                            self.evaluation_row_index
                                - self.identity_subpolynomial_evaluations.len()
                                + 1,
                        ));
                }
                if degree + 1 > self.subpolynomial_max_multiplicands {
                    Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                }
                self.identity_subpolynomial_evaluations[self.evaluation_row_index].push(eval);
            }
            SumcheckSubpolynomialType::ZeroSum => {
                if self.zerosum_subpolynomial_evaluations.len() <= self.evaluation_row_index {
                    self.zerosum_subpolynomial_evaluations
                        .extend(iter::repeat(Vec::new()).take(
                            self.evaluation_row_index
                                - self.zerosum_subpolynomial_evaluations.len()
                                + 1,
                        ));
                }
                if degree > self.subpolynomial_max_multiplicands {
                    Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                }
                self.zerosum_subpolynomial_evaluations[self.evaluation_row_index].push(eval);
            }
        }
        Ok(())
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        let res = self
            .bit_distributions
            .get(self.bit_distribution_offset)
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewBitDistributions)?;
        self.bit_distribution_offset += 1;
        Ok(res)
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_final_round_pcs_proof_mles;
        self.consumed_final_round_pcs_proof_mles += 1;
        Ok(*self
            .final_round_mles
            .get(self.evaluation_row_index)
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)?
            .get(index)
            .unwrap_or(&S::ZERO))
    }

    fn singleton_chi_evaluation(&self) -> S {
        unimplemented!("No tests currently use this function")
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        iter::repeat_with(|| self.try_consume_final_round_mle_evaluation())
            .take(count)
            .collect()
    }
}

impl<S: Scalar> MockVerificationBuilder<S> {
    #[allow(
        clippy::missing_panics_doc,
        reason = "The only possible panic is from the assertion comparing lengths, which is clear from context."
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bit_distributions: Vec<BitDistribution>,
        subpolynomial_max_multiplicands: usize,
        final_round_mles: Vec<Vec<S>>,
    ) -> Self {
        Self {
            bit_distributions,
            bit_distribution_offset: 0,
            consumed_final_round_pcs_proof_mles: 0,
            subpolynomial_max_multiplicands,
            evaluation_row_index: 0,
            final_round_mles,
            identity_subpolynomial_evaluations: Vec::new(),
            zerosum_subpolynomial_evaluations: Vec::new(),
        }
    }

    pub fn increment_row_index(&mut self) {
        self.bit_distribution_offset = 0;
        self.consumed_final_round_pcs_proof_mles = 0;
        self.evaluation_row_index += 1;
    }
}
