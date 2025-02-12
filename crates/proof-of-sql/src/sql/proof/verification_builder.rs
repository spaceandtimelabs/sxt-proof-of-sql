use super::{SumcheckMleEvaluations, SumcheckSubpolynomialType};
use crate::base::{bit::BitDistribution, proof::ProofSizeMismatch, scalar::Scalar};
use alloc::{collections::VecDeque, vec::Vec};
use core::iter;

pub trait VerificationBuilder<S: Scalar> {
    /// Consume the evaluation of a chi evaluation
    fn try_consume_chi_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume the evaluation of a rho evaluation
    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume the evaluation of a first round MLE used in sumcheck and provide the commitment of the MLE
    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume the evaluation of a final round MLE used in sumcheck and provide the commitment of the MLE
    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume multiple final round MLE evaluations
    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch>;

    /// Consume a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch>;

    /// Produce the evaluation of a subpolynomial used in sumcheck
    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch>;

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Retrieves the `singleton_chi_evaluation` from the `mle_evaluations`
    fn singleton_chi_evaluation(&self) -> S;

    /// Retrieves the `rho_256_evaluation` from the `mle_evaluations`
    fn rho_256_evaluation(&self) -> Option<S>;
}

/// Track components used to verify a query's proof
pub struct StandardVerificationBuilder<'a, S: Scalar> {
    mle_evaluations: SumcheckMleEvaluations<'a, S>,
    subpolynomial_multipliers: &'a [S],
    sumcheck_evaluation: S,
    bit_distributions: &'a [BitDistribution],
    consumed_chi_evaluations: usize,
    consumed_rho_evaluations: usize,
    consumed_first_round_pcs_proof_mles: usize,
    consumed_final_round_pcs_proof_mles: usize,
    produced_subpolynomials: usize,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: VecDeque<S>,
    chi_evaluation_length_queue: Vec<usize>,
    rho_evaluation_length_queue: Vec<usize>,
    subpolynomial_max_multiplicands: usize,
}

impl<'a, S: Scalar> StandardVerificationBuilder<'a, S> {
    #[allow(
        clippy::missing_panics_doc,
        reason = "The only possible panic is from the assertion comparing lengths, which is clear from context."
    )]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mle_evaluations: SumcheckMleEvaluations<'a, S>,
        bit_distributions: &'a [BitDistribution],
        subpolynomial_multipliers: &'a [S],
        post_result_challenges: VecDeque<S>,
        chi_evaluation_length_queue: Vec<usize>,
        rho_evaluation_length_queue: Vec<usize>,
        subpolynomial_max_multiplicands: usize,
    ) -> Self {
        Self {
            mle_evaluations,
            bit_distributions,
            subpolynomial_multipliers,
            sumcheck_evaluation: S::zero(),
            consumed_chi_evaluations: 0,
            consumed_rho_evaluations: 0,
            consumed_first_round_pcs_proof_mles: 0,
            consumed_final_round_pcs_proof_mles: 0,
            produced_subpolynomials: 0,
            post_result_challenges,
            chi_evaluation_length_queue,
            rho_evaluation_length_queue,
            subpolynomial_max_multiplicands,
        }
    }

    #[allow(
        clippy::missing_panics_doc,
        reason = "The panic condition is clear due to the assertion that checks if the computation is completed."
    )]
    /// Get the evaluation of the sumcheck polynomial at its randomly selected point
    pub fn sumcheck_evaluation(&self) -> S {
        assert!(self.completed());
        self.sumcheck_evaluation
    }

    /// Check that the verification builder is completely built up
    fn completed(&self) -> bool {
        self.bit_distributions.is_empty()
            && self.produced_subpolynomials == self.subpolynomial_multipliers.len()
            && self.consumed_first_round_pcs_proof_mles
                == self.mle_evaluations.first_round_pcs_proof_evaluations.len()
            && self.consumed_final_round_pcs_proof_mles
                == self.mle_evaluations.final_round_pcs_proof_evaluations.len()
            && self.post_result_challenges.is_empty()
    }
}

impl<S: Scalar> VerificationBuilder<S> for StandardVerificationBuilder<'_, S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_chi_evaluations;
        let length = self
            .chi_evaluation_length_queue
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewChiLengths)?;
        self.consumed_chi_evaluations += 1;
        Ok(*self
            .mle_evaluations
            .chi_evaluations
            .get(&length)
            .ok_or(ProofSizeMismatch::ChiLengthNotFound)?)
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_rho_evaluations;
        let length = self
            .rho_evaluation_length_queue
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewRhoLengths)?;
        self.consumed_rho_evaluations += 1;
        Ok(*self
            .mle_evaluations
            .rho_evaluations
            .get(&length)
            .ok_or(ProofSizeMismatch::RhoLengthNotFound)?)
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_first_round_pcs_proof_mles;
        self.consumed_first_round_pcs_proof_mles += 1;
        self.mle_evaluations
            .first_round_pcs_proof_evaluations
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_final_round_pcs_proof_mles;
        self.consumed_final_round_pcs_proof_mles += 1;
        self.mle_evaluations
            .final_round_pcs_proof_evaluations
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        iter::repeat_with(|| self.try_consume_final_round_mle_evaluation())
            .take(count)
            .collect()
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        let res = self
            .bit_distributions
            .first()
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewBitDistributions)?;
        self.bit_distributions = &self.bit_distributions[1..];
        Ok(res)
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        self.sumcheck_evaluation += self
            .subpolynomial_multipliers
            .get(self.produced_subpolynomials)
            .copied()
            .ok_or(ProofSizeMismatch::ConstraintCountMismatch)?
            * match subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    if degree + 1 > self.subpolynomial_max_multiplicands {
                        Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                    }
                    eval * self.mle_evaluations.random_evaluation
                }
                SumcheckSubpolynomialType::ZeroSum => {
                    if degree > self.subpolynomial_max_multiplicands {
                        Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                    }
                    eval
                }
            };
        self.produced_subpolynomials += 1;
        Ok(())
    }

    /// # Panics
    /// This function will panic if there are no post-result challenges available to pop from the stack.
    ///
    /// # Panics
    /// This function will panic if `post_result_challenges` is empty,
    /// as it attempts to pop an element from the vector and unwraps the result.
    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        self.post_result_challenges
            .pop_front()
            .ok_or(ProofSizeMismatch::PostResultCountMismatch)
    }

    fn singleton_chi_evaluation(&self) -> S {
        self.mle_evaluations.singleton_chi_evaluation
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        self.mle_evaluations.rho_256_evaluation
    }
}
