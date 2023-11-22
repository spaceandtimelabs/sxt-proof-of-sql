use super::{
    CompositePolynomialBuilder, MultilinearExtension, MultilinearExtensionImpl,
    SumcheckRandomScalars, SumcheckSubpolynomial,
};
use crate::base::{
    bit::BitDistribution, polynomial::CompositePolynomial, scalar::ArkScalar, slice_ops,
};
use blitzar::{compute::compute_commitments, sequence::Sequence};
use bumpalo::Bump;
use curve25519_dalek::{ristretto::CompressedRistretto, traits::Identity};
use num_traits::Zero;

/// Track components used to form a query's proof
pub struct ProofBuilder<'a> {
    table_length: usize,
    num_sumcheck_variables: usize,
    bit_distributions: Vec<BitDistribution>,
    commitment_descriptor: Vec<Sequence<'a>>,
    pre_result_mles: Vec<Box<dyn MultilinearExtension + 'a>>,
    sumcheck_subpolynomials: Vec<SumcheckSubpolynomial<'a>>,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: Vec<ArkScalar>,
}

impl<'a> ProofBuilder<'a> {
    #[tracing::instrument(name = "proofs.sql.proof.proof_builder.new", level = "debug", skip_all)]
    pub fn new(
        table_length: usize,
        num_sumcheck_variables: usize,
        post_result_challenges: Vec<ArkScalar>,
    ) -> Self {
        Self {
            table_length,
            num_sumcheck_variables,
            bit_distributions: Vec::new(),
            commitment_descriptor: Vec::new(),
            pre_result_mles: Vec::new(),
            sumcheck_subpolynomials: Vec::new(),
            post_result_challenges,
        }
    }

    pub fn table_length(&self) -> usize {
        self.table_length
    }

    pub fn num_sumcheck_variables(&self) -> usize {
        self.num_sumcheck_variables
    }

    pub fn num_sumcheck_subpolynomials(&self) -> usize {
        self.sumcheck_subpolynomials.len()
    }

    /// Produce a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    pub fn produce_bit_distribution(&mut self, dist: BitDistribution) {
        self.bit_distributions.push(dist);
    }

    /// Produce an anchored MLE that we can reference in sumcheck.
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.produce_anchored_mle",
        level = "debug",
        skip_all
    )]
    pub fn produce_anchored_mle<T: Sync>(&mut self, data: &'a [T])
    where
        &'a T: Into<ArkScalar>,
    {
        self.pre_result_mles
            .push(Box::new(MultilinearExtensionImpl::new(data)));
    }

    /// Produce an MLE for a intermediate computed column that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.produce_intermediate_mle",
        level = "debug",
        skip_all
    )]
    pub fn produce_intermediate_mle<T: Sync>(&mut self, data: &'a [T])
    where
        &'a [T]: Into<Sequence<'a>>,
        &'a T: Into<ArkScalar>,
    {
        self.commitment_descriptor.push(data.into());
        self.produce_anchored_mle(data);
    }

    /// Produce an MLE for a intermediate computed column of `ArkScalar`s that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    ///
    /// This method differs from `produce_intermediate_mle` in that it takes `ArkScalar`s as input. This is needed because a
    /// slice of `ArkScalar`s does not implement `Into<DenseSequence>`.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.produce_intermediate_mle_from_ark_scalars",
        level = "debug",
        skip_all
    )]
    pub fn produce_intermediate_mle_from_ark_scalars(
        &mut self,
        data: &'a [ArkScalar],
        alloc: &'a Bump,
    ) {
        let cast_data: &mut [[u64; 4]] = alloc.alloc_slice_fill_default(data.len());
        slice_ops::slice_cast_mut(data, cast_data);
        self.commitment_descriptor.push(cast_data.into());
        self.produce_anchored_mle(data);
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.produce_sumcheck_subpolynomial",
        level = "debug",
        skip_all
    )]
    pub fn produce_sumcheck_subpolynomial(&mut self, group: SumcheckSubpolynomial<'a>) {
        self.sumcheck_subpolynomials.push(group);
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.commit_intermediate_mles",
        level = "info",
        skip_all
    )]
    pub fn commit_intermediate_mles(&self, offset_generators: usize) -> Vec<CompressedRistretto> {
        let mut res = vec![CompressedRistretto::identity(); self.commitment_descriptor.len()];
        compute_commitments(
            &mut res,
            &self.commitment_descriptor,
            offset_generators as u64,
        );
        res
    }

    /// Given random multipliers, construct an aggregatated sumcheck polynomial from all
    /// the individual subpolynomials.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.make_sumcheck_polynomial",
        level = "info",
        skip_all
    )]
    pub fn make_sumcheck_polynomial(&self, scalars: &SumcheckRandomScalars) -> CompositePolynomial {
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
        name = "proofs.sql.proof.proof_builder.evaluate_pre_result_mles",
        level = "info",
        skip_all
    )]
    pub fn evaluate_pre_result_mles(&self, evaluation_vec: &[ArkScalar]) -> Vec<ArkScalar> {
        let mut res = Vec::with_capacity(self.pre_result_mles.len());
        for evaluator in self.pre_result_mles.iter() {
            res.push(evaluator.inner_product(evaluation_vec));
        }
        res
    }

    /// Given random multipliers, multiply and add together all of the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.fold_pre_result_mles",
        level = "info",
        skip_all
    )]
    pub fn fold_pre_result_mles(&self, multipliers: &[ArkScalar]) -> Vec<ArkScalar> {
        assert_eq!(multipliers.len(), self.pre_result_mles.len());
        let mut res = vec![Zero::zero(); self.table_length];
        for (multiplier, evaluator) in multipliers.iter().zip(self.pre_result_mles.iter()) {
            evaluator.mul_add(&mut res, multiplier);
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
    pub fn consume_post_result_challenge(&mut self) -> ArkScalar {
        self.post_result_challenges.pop().unwrap()
    }
}
