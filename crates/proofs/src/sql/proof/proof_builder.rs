use super::{
    make_sumcheck_term, IntermediateQueryResult, IntermediateResultColumn, MultilinearExtension,
    MultilinearExtensionImpl, ProofCounts, SumcheckSubpolynomial,
};
use crate::base::polynomial::CompositePolynomial;
use crate::base::scalar::IntoScalar;
use pedersen::compute::compute_commitments;
use pedersen::sequences::{DenseSequence, Sequence};

use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar, traits::Identity};

/// Track components used to form a query's proof
#[allow(dead_code)]
pub struct ProofBuilder<'a> {
    num_sumcheck_variables: usize,
    result_index_vector: &'a [u64],
    result_columns: Vec<Box<dyn IntermediateResultColumn + 'a>>,
    commitment_descriptor: Vec<Sequence<'a>>,
    pre_result_mles: Vec<Box<dyn MultilinearExtension + 'a>>,
    sumcheck_subpolynomials: Vec<SumcheckSubpolynomial>,
}

impl<'a> ProofBuilder<'a> {
    pub fn new(counts: &ProofCounts) -> Self {
        Self {
            num_sumcheck_variables: counts.sumcheck_variables,
            result_index_vector: &[],
            result_columns: Vec::with_capacity(counts.result_columns),
            commitment_descriptor: Vec::with_capacity(counts.anchored_mles),
            pre_result_mles: Vec::with_capacity(counts.anchored_mles + counts.intermediate_mles),
            sumcheck_subpolynomials: Vec::with_capacity(counts.sumcheck_subpolynomials),
        }
    }

    /// Produce an anchored MLE that we can reference in sumcheck.
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment.
    pub fn produce_anchored_mle<T: IntoScalar>(&mut self, data: &'a [T]) {
        assert!(self.pre_result_mles.len() < self.pre_result_mles.capacity());
        self.pre_result_mles
            .push(Box::new(MultilinearExtensionImpl::new(data)));
    }

    /// Produce an MLE for a intermediate computed column that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    pub fn produce_intermediate_mle<T: IntoScalar>(&mut self, data: &'a [T]) {
        assert!(self.commitment_descriptor.len() < self.commitment_descriptor.capacity());
        let len = data.len() * std::mem::size_of::<T>();
        let slice = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, len) };
        self.commitment_descriptor
            .push(Sequence::Dense(DenseSequence {
                data_slice: slice,
                element_size: std::mem::size_of::<T>(),
            }));
        self.produce_anchored_mle(data);
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn produce_sumcheck_subpolynomial(&mut self, group: SumcheckSubpolynomial) {
        assert!(self.sumcheck_subpolynomials.len() < self.sumcheck_subpolynomials.capacity());
        self.sumcheck_subpolynomials.push(group);
    }

    /// Set the indexes of the rows select in the result
    pub fn set_result_indexes(&mut self, result_index_vector: &'a [u64]) {
        self.result_index_vector = result_index_vector;
    }

    /// Produce an intermediate result column that will be sent to the verifier.
    pub fn produce_result_column(&mut self, col: Box<dyn IntermediateResultColumn + 'a>) {
        assert!(self.result_columns.len() < self.result_columns.capacity());
        self.result_columns.push(col);
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    pub fn commit_intermediate_mles(&self) -> Vec<CompressedRistretto> {
        assert_eq!(
            self.commitment_descriptor.len(),
            self.commitment_descriptor.capacity()
        );
        let mut res = vec![CompressedRistretto::identity(); self.commitment_descriptor.len()];
        compute_commitments(&mut res, &self.commitment_descriptor);
        res
    }

    /// Construct the intermediate query result to be sent to the verifier.
    pub fn make_intermediate_query_result(&self) -> IntermediateQueryResult {
        IntermediateQueryResult::new(self.result_index_vector, &self.result_columns)
    }

    /// Given random multipliers, construct an aggregatated sumcheck polynomial from all
    /// the individual subpolynomials.
    pub fn make_sumcheck_polynomial(&self, multipliers: &[Scalar]) -> CompositePolynomial {
        assert_eq!(
            self.sumcheck_subpolynomials.len(),
            self.sumcheck_subpolynomials.capacity()
        );
        let n = 1 << self.num_sumcheck_variables;
        assert_eq!(multipliers.len(), n + self.sumcheck_subpolynomials.len());
        let (entry_multipliers, group_multipliers) = multipliers.split_at(n);
        let mut res = CompositePolynomial::new(self.num_sumcheck_variables);
        let fr = make_sumcheck_term(self.num_sumcheck_variables, entry_multipliers);
        for (multiplier, subpoly) in group_multipliers
            .iter()
            .zip(self.sumcheck_subpolynomials.iter())
        {
            subpoly.mul_add(&mut res, fr.clone(), *multiplier);
        }
        res
    }

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    pub fn evaluate_pre_result_mles(&self, evaluation_vec: &[Scalar]) -> Vec<Scalar> {
        assert_eq!(self.pre_result_mles.len(), self.pre_result_mles.capacity());
        let mut res = Vec::with_capacity(self.pre_result_mles.len());
        for evaluator in self.pre_result_mles.iter() {
            res.push(evaluator.evaluate(evaluation_vec));
        }
        res
    }

    /// Given random multipliers, multiply and add together all of the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    pub fn fold_pre_result_mles(&self, multipliers: &[Scalar]) -> Vec<Scalar> {
        assert_eq!(self.pre_result_mles.len(), self.pre_result_mles.capacity());
        assert_eq!(multipliers.len(), self.pre_result_mles.len());
        let mut res = vec![Scalar::zero(); 1 << self.num_sumcheck_variables];
        for (multiplier, evaluator) in multipliers.iter().zip(self.pre_result_mles.iter()) {
            evaluator.mul_add(&mut res, multiplier);
        }
        res
    }
}
