use super::{
    CompositePolynomialBuilder, MultilinearExtension, MultilinearExtensionImpl, ProofCounts,
    ProvableQueryResult, ProvableResultColumn, SumcheckRandomScalars, SumcheckSubpolynomial,
};
use crate::base::polynomial::CompositePolynomial;
use crate::base::scalar::ArkScalar;
use crate::base::slice_ops;
use blitzar::compute::compute_commitments;
use blitzar::sequence::Sequence;
use bumpalo::Bump;
use num_traits::Zero;

use curve25519_dalek::{ristretto::CompressedRistretto, traits::Identity};

/// Track components used to form a query's proof
#[allow(dead_code)]
pub struct ProofBuilder<'a> {
    table_length: usize,
    num_sumcheck_variables: usize,
    result_index_vector: &'a [u64],
    result_columns: Vec<Box<dyn ProvableResultColumn + 'a>>,
    commitment_descriptor: Vec<Sequence<'a>>,
    pre_result_mles: Vec<Box<dyn MultilinearExtension + 'a>>,
    sumcheck_subpolynomials: Vec<SumcheckSubpolynomial<'a>>,
}

impl<'a> ProofBuilder<'a> {
    #[tracing::instrument(name = "proofs.sql.proof.proof_builder.new", level = "debug", skip_all)]
    pub fn new(counts: &ProofCounts) -> Self {
        Self {
            table_length: counts.table_length,
            num_sumcheck_variables: counts.sumcheck_variables,
            result_index_vector: &[],
            result_columns: Vec::with_capacity(counts.result_columns),
            commitment_descriptor: Vec::with_capacity(counts.intermediate_mles),
            pre_result_mles: Vec::with_capacity(counts.anchored_mles + counts.intermediate_mles),
            sumcheck_subpolynomials: Vec::with_capacity(counts.sumcheck_subpolynomials),
        }
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
        assert!(self.pre_result_mles.len() < self.pre_result_mles.capacity());
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
        assert!(self.commitment_descriptor.len() < self.commitment_descriptor.capacity());
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
        assert!(self.commitment_descriptor.len() < self.commitment_descriptor.capacity());
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
        assert!(self.sumcheck_subpolynomials.len() < self.sumcheck_subpolynomials.capacity());
        self.sumcheck_subpolynomials.push(group);
    }

    /// Set the indexes of the rows select in the result
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.set_result_indexes",
        level = "debug",
        skip_all
    )]
    pub fn set_result_indexes(&mut self, result_index_vector: &'a [u64]) {
        self.result_index_vector = result_index_vector;
    }

    /// Produce an intermediate result column that will be sent to the verifier.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.produce_result_column",
        level = "debug",
        skip_all
    )]
    pub fn produce_result_column(&mut self, col: Box<dyn ProvableResultColumn + 'a>) {
        assert!(self.result_columns.len() < self.result_columns.capacity());
        self.result_columns.push(col);
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.commit_intermediate_mles",
        level = "info",
        skip_all
    )]
    pub fn commit_intermediate_mles(&self, offset_generators: usize) -> Vec<CompressedRistretto> {
        assert_eq!(
            self.commitment_descriptor.len(),
            self.commitment_descriptor.capacity()
        );
        let mut res = vec![CompressedRistretto::identity(); self.commitment_descriptor.len()];
        compute_commitments(
            &mut res,
            &self.commitment_descriptor,
            offset_generators as u64,
        );
        res
    }

    /// Construct the intermediate query result to be sent to the verifier.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.make_provable_query_result",
        level = "debug",
        skip_all
    )]
    pub fn make_provable_query_result(&self) -> ProvableQueryResult {
        ProvableQueryResult::new(self.result_index_vector, &self.result_columns)
    }

    /// Given random multipliers, construct an aggregatated sumcheck polynomial from all
    /// the individual subpolynomials.
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_builder.make_sumcheck_polynomial",
        level = "info",
        skip_all
    )]
    pub fn make_sumcheck_polynomial(&self, scalars: &SumcheckRandomScalars) -> CompositePolynomial {
        assert_eq!(
            self.sumcheck_subpolynomials.len(),
            self.sumcheck_subpolynomials.capacity()
        );
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
        assert_eq!(self.pre_result_mles.len(), self.pre_result_mles.capacity());
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
        assert_eq!(self.pre_result_mles.len(), self.pre_result_mles.capacity());
        assert_eq!(multipliers.len(), self.pre_result_mles.len());
        let mut res = vec![Zero::zero(); self.table_length];
        for (multiplier, evaluator) in multipliers.iter().zip(self.pre_result_mles.iter()) {
            evaluator.mul_add(&mut res, multiplier);
        }
        res
    }
}
