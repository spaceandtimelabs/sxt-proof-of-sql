use crate::base::scalar::ArkScalar;

use super::SumcheckMleEvaluations;

use crate::base::bit::BitDistribution;
use curve25519_dalek::{ristretto::RistrettoPoint, traits::Identity};
use num_traits::Zero;

/// Track components used to verify a query's proof
pub struct VerificationBuilder<'a> {
    pub mle_evaluations: SumcheckMleEvaluations<'a>,
    generator_offset: usize,
    intermediate_commitments: &'a [RistrettoPoint],
    subpolynomial_multipliers: &'a [ArkScalar],
    inner_product_multipliers: &'a [ArkScalar],
    sumcheck_evaluation: ArkScalar,
    bit_distributions: &'a [BitDistribution],
    folded_pre_result_commitment: RistrettoPoint,
    folded_pre_result_evaluation: ArkScalar,
    consumed_result_mles: usize,
    consumed_pre_result_mles: usize,
    consumed_intermediate_mles: usize,
    produced_subpolynomials: usize,
}

impl<'a> VerificationBuilder<'a> {
    pub fn new(
        generator_offset: usize,
        mle_evaluations: SumcheckMleEvaluations<'a>,
        bit_distributions: &'a [BitDistribution],
        intermediate_commitments: &'a [RistrettoPoint],
        subpolynomial_multipliers: &'a [ArkScalar],
        inner_product_multipliers: &'a [ArkScalar],
    ) -> Self {
        assert_eq!(
            inner_product_multipliers.len(),
            mle_evaluations.pre_result_evaluations.len()
        );
        Self {
            mle_evaluations,
            generator_offset,
            bit_distributions,
            intermediate_commitments,
            subpolynomial_multipliers,
            inner_product_multipliers,
            sumcheck_evaluation: ArkScalar::zero(),
            folded_pre_result_commitment: RistrettoPoint::identity(),
            folded_pre_result_evaluation: ArkScalar::zero(),
            consumed_result_mles: 0,
            consumed_pre_result_mles: 0,
            consumed_intermediate_mles: 0,
            produced_subpolynomials: 0,
        }
    }

    pub fn table_length(&self) -> usize {
        self.mle_evaluations.table_length
    }

    pub fn generator_offset(&self) -> usize {
        self.generator_offset
    }

    /// Consume the evaluation of an anchored MLE used in sumcheck and provide the commitment of the MLE
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment
    pub fn consume_anchored_mle(&mut self, commitment: &RistrettoPoint) -> ArkScalar {
        let index = self.consumed_pre_result_mles;
        let multiplier = self.inner_product_multipliers[index];
        self.folded_pre_result_commitment += multiplier * commitment;
        self.consumed_pre_result_mles += 1;
        let res = self.mle_evaluations.pre_result_evaluations[index];
        self.folded_pre_result_evaluation += multiplier * res;
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
    pub fn consume_intermediate_mle(&mut self) -> ArkScalar {
        let commitment = &self.intermediate_commitments[self.consumed_intermediate_mles];
        self.consumed_intermediate_mles += 1;
        self.consume_anchored_mle(commitment)
    }

    /// Consume the evaluation of the MLE for a result column used in sumcheck
    pub fn consume_result_mle(&mut self) -> ArkScalar {
        let index = self.consumed_result_mles;
        self.consumed_result_mles += 1;
        self.mle_evaluations.result_evaluations[index]
    }

    /// Produce the evaluation of a subpolynomial used in sumcheck
    pub fn produce_sumcheck_subpolynomial_evaluation(&mut self, eval: &ArkScalar) {
        self.sumcheck_evaluation +=
            self.subpolynomial_multipliers[self.produced_subpolynomials] * *eval;
        self.produced_subpolynomials += 1;
    }

    /// Get the evaluation of the sumcheck polynomial at its randomly selected point
    pub fn sumcheck_evaluation(&self) -> ArkScalar {
        assert!(self.completed());
        self.sumcheck_evaluation
    }

    /// Get the commitment of the folded pre-result MLE vectors used in a verifiable query's
    /// bulletproof
    pub fn folded_pre_result_commitment(&self) -> RistrettoPoint {
        assert!(self.completed());
        self.folded_pre_result_commitment
    }

    /// Get the evaluation of the folded pre-result MLE vectors used in a verifiable query's
    /// bulletproof
    pub fn folded_pre_result_evaluation(&self) -> ArkScalar {
        assert!(self.completed());
        self.folded_pre_result_evaluation
    }

    /// Check that the verification builder is completely built up
    fn completed(&self) -> bool {
        self.bit_distributions.is_empty()
            && self.produced_subpolynomials == self.subpolynomial_multipliers.len()
            && self.consumed_intermediate_mles == self.intermediate_commitments.len()
            && self.consumed_pre_result_mles == self.mle_evaluations.pre_result_evaluations.len()
            && self.consumed_result_mles == self.mle_evaluations.result_evaluations.len()
    }
}
