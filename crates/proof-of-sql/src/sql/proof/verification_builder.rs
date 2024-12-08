use super::{SumcheckMleEvaluations, SumcheckSubpolynomialType};
use crate::base::{bit::BitDistribution, scalar::Scalar};
use alloc::vec::Vec;

/// Track components used to verify a query's proof
pub struct VerificationBuilder<'a, S: Scalar> {
    pub mle_evaluations: SumcheckMleEvaluations<'a, S>,
    generator_offset: usize,
    subpolynomial_multipliers: &'a [S],
    inner_product_multipliers: &'a [S],
    sumcheck_evaluation: S,
    bit_distributions: &'a [BitDistribution],
    folded_pcs_proof_evaluation: S,
    consumed_one_evaluations: usize,
    consumed_pcs_proof_mles: usize,
    consumed_intermediate_mles: usize,
    produced_subpolynomials: usize,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: Vec<S>,
    one_evaluation_length_queue: Vec<usize>,
}

impl<'a, S: Scalar> VerificationBuilder<'a, S> {
    #[allow(
        clippy::missing_panics_doc,
        reason = "The only possible panic is from the assertion comparing lengths, which is clear from context."
    )]
    pub fn new(
        generator_offset: usize,
        mle_evaluations: SumcheckMleEvaluations<'a, S>,
        bit_distributions: &'a [BitDistribution],
        subpolynomial_multipliers: &'a [S],
        inner_product_multipliers: &'a [S],
        post_result_challenges: Vec<S>,
        one_evaluation_length_queue: Vec<usize>,
    ) -> Self {
        assert_eq!(
            inner_product_multipliers.len(),
            mle_evaluations.pcs_proof_evaluations.len()
        );
        Self {
            mle_evaluations,
            generator_offset,
            bit_distributions,
            subpolynomial_multipliers,
            inner_product_multipliers,
            sumcheck_evaluation: S::zero(),
            folded_pcs_proof_evaluation: S::zero(),
            consumed_one_evaluations: 0,
            consumed_pcs_proof_mles: 0,
            consumed_intermediate_mles: 0,
            produced_subpolynomials: 0,
            post_result_challenges,
            one_evaluation_length_queue,
        }
    }

    /// Consume the evaluation of a one evaluation
    ///
    /// # Panics
    /// It should never panic, as the length of the one evaluation is guaranteed to be present
    pub fn consume_one_evaluation(&mut self) -> S {
        let index = self.consumed_one_evaluations;
        let length = self.one_evaluation_length_queue[index];
        self.consumed_one_evaluations += 1;
        *self
            .mle_evaluations
            .one_evaluations
            .get(&length)
            .expect("One evaluation not found")
    }

    pub fn generator_offset(&self) -> usize {
        self.generator_offset
    }

    /// Consume the evaluation of an anchored MLE used in sumcheck and provide the commitment of the MLE
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment
    pub fn consume_anchored_mle(&mut self) -> S {
        let index = self.consumed_pcs_proof_mles;
        let multiplier = self.inner_product_multipliers[index];
        self.consumed_pcs_proof_mles += 1;
        let res = self.mle_evaluations.pcs_proof_evaluations[index];
        self.folded_pcs_proof_evaluation += multiplier * res;
        res
    }

    /// Consume a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    pub fn consume_bit_distribution(&mut self) -> BitDistribution {
        let res = self.bit_distributions[0].clone();
        self.bit_distributions = &self.bit_distributions[1..];
        res
    }

    /// Consume the evaluation of an intermediate MLE used in sumcheck
    ///
    /// An interemdiate MLE is one where the verifier doesn't have access to its commitment
    pub fn consume_intermediate_mle(&mut self) -> S {
        self.consumed_intermediate_mles += 1;
        self.consume_anchored_mle()
    }

    /// Produce the evaluation of a subpolynomial used in sumcheck
    pub fn produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
    ) {
        self.sumcheck_evaluation += self.subpolynomial_multipliers[self.produced_subpolynomials]
            * match subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    eval * self.mle_evaluations.random_evaluation
                }
                SumcheckSubpolynomialType::ZeroSum => eval,
            };
        self.produced_subpolynomials += 1;
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

    #[allow(
        clippy::missing_panics_doc,
        reason = "Panic conditions are self-evident from the completed assertion."
    )]
    /// Get folding factors for the pre-result commitments
    pub fn inner_product_multipliers(&self) -> &[S] {
        assert!(self.completed());
        self.inner_product_multipliers
    }

    #[allow(
        clippy::missing_panics_doc,
        reason = "The panic condition is evident due to the assertion ensuring completion."
    )]
    /// Get the evaluation of the folded pre-result MLE vectors used in a verifiable query's
    /// bulletproof
    pub fn folded_pcs_proof_evaluation(&self) -> S {
        assert!(self.completed());
        self.folded_pcs_proof_evaluation
    }

    /// Check that the verification builder is completely built up
    fn completed(&self) -> bool {
        self.bit_distributions.is_empty()
            && self.produced_subpolynomials == self.subpolynomial_multipliers.len()
            && self.consumed_pcs_proof_mles == self.mle_evaluations.pcs_proof_evaluations.len()
            && self.post_result_challenges.is_empty()
    }

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// # Panics
    /// This function will panic if there are no post-result challenges available to pop from the stack.
    ///
    /// # Panics
    /// This function will panic if `post_result_challenges` is empty,
    /// as it attempts to pop an element from the vector and unwraps the result.
    pub fn consume_post_result_challenge(&mut self) -> S {
        self.post_result_challenges.pop().unwrap()
    }
}
