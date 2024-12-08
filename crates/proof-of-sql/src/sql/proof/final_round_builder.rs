use super::{
    CompositePolynomialBuilder, SumcheckRandomScalars, SumcheckSubpolynomial,
    SumcheckSubpolynomialTerm, SumcheckSubpolynomialType,
};
use crate::base::{
    bit::BitDistribution,
    commitment::{Commitment, CommittableColumn, VecCommitmentExt},
    polynomial::{CompositePolynomial, MultilinearExtension},
    scalar::Scalar,
};
use alloc::{boxed::Box, vec::Vec};

/// Track components used to form a query's proof
pub struct FinalRoundBuilder<'a, S: Scalar> {
    num_sumcheck_variables: usize,
    bit_distributions: Vec<BitDistribution>,
    commitment_descriptor: Vec<CommittableColumn<'a>>,
    pcs_proof_mles: Vec<Box<dyn MultilinearExtension<S> + 'a>>,
    sumcheck_subpolynomials: Vec<SumcheckSubpolynomial<'a, S>>,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: Vec<S>,
}

impl<'a, S: Scalar> FinalRoundBuilder<'a, S> {
    pub fn new(num_sumcheck_variables: usize, post_result_challenges: Vec<S>) -> Self {
        Self {
            num_sumcheck_variables,
            bit_distributions: Vec::new(),
            commitment_descriptor: Vec::new(),
            pcs_proof_mles: Vec::new(),
            sumcheck_subpolynomials: Vec::new(),
            post_result_challenges,
        }
    }

    pub fn num_sumcheck_variables(&self) -> usize {
        self.num_sumcheck_variables
    }

    pub fn num_sumcheck_subpolynomials(&self) -> usize {
        self.sumcheck_subpolynomials.len()
    }

    pub fn pcs_proof_mles(&self) -> &[Box<dyn MultilinearExtension<S> + 'a>] {
        &self.pcs_proof_mles
    }

    /// Produce a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    pub fn produce_bit_distribution(&mut self, dist: BitDistribution) {
        self.bit_distributions.push(dist);
    }

    /// Produce an anchored MLE that we can reference in sumcheck.
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment.
    pub fn produce_anchored_mle(&mut self, data: impl MultilinearExtension<S> + 'a) {
        self.pcs_proof_mles.push(Box::new(data));
    }

    /// Produce an MLE for a intermediate computed column that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    pub fn produce_intermediate_mle(
        &mut self,
        data: impl MultilinearExtension<S> + Into<CommittableColumn<'a>> + Copy + 'a,
    ) {
        self.commitment_descriptor.push(data.into());
        self.produce_anchored_mle(data);
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn produce_sumcheck_subpolynomial(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    ) {
        self.sumcheck_subpolynomials
            .push(SumcheckSubpolynomial::new(subpolynomial_type, terms));
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    #[tracing::instrument(
        name = "FinalRoundBuilder::commit_intermediate_mles",
        level = "debug",
        skip_all
    )]
    pub fn commit_intermediate_mles<C: Commitment>(
        &self,
        offset_generators: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Vec<C> {
        Vec::from_commitable_columns_with_offset(
            &self.commitment_descriptor,
            offset_generators,
            setup,
        )
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn sumcheck_subpolynomials(&self) -> &[SumcheckSubpolynomial<'a, S>] {
        &self.sumcheck_subpolynomials
    }

    /// Given random multipliers, construct an aggregatated sumcheck polynomial from all
    /// the individual subpolynomials.
    #[tracing::instrument(
        name = "FinalRoundBuilder::make_sumcheck_polynomial",
        level = "debug",
        skip_all
    )]
    pub fn make_sumcheck_polynomial(
        &self,
        scalars: &SumcheckRandomScalars<S>,
    ) -> CompositePolynomial<S> {
        let mut builder = CompositePolynomialBuilder::new(
            self.num_sumcheck_variables,
            &scalars.compute_entrywise_multipliers(),
        );
        for (multiplier, subpoly) in scalars
            .subpolynomial_multipliers
            .iter()
            .zip(self.sumcheck_subpolynomials.iter())
        {
            subpoly.compose(&mut builder, *multiplier);
        }
        builder.make_composite_polynomial()
    }

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "FinalRoundBuilder::evaluate_pcs_proof_mles",
        level = "debug",
        skip_all
    )]
    pub fn evaluate_pcs_proof_mles(&self, evaluation_vec: &[S]) -> Vec<S> {
        let mut res = Vec::with_capacity(self.pcs_proof_mles.len());
        for evaluator in &self.pcs_proof_mles {
            res.push(evaluator.inner_product(evaluation_vec));
        }
        res
    }

    pub fn bit_distributions(&self) -> &[BitDistribution] {
        &self.bit_distributions
    }

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    /// # Panics
    ///
    /// Will panic if there are no post-result challenges available to pop from the stack.
    pub fn consume_post_result_challenge(&mut self) -> S {
        self.post_result_challenges.pop().unwrap()
    }
}
