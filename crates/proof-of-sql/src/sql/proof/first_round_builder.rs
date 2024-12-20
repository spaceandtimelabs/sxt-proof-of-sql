use crate::{
    base::{
        commitment::{Commitment, CommittableColumn, VecCommitmentExt},
        polynomial::MultilinearExtension,
        scalar::Scalar,
    },
    utils::log,
};
use alloc::{boxed::Box, vec::Vec};
/// Track the result created by a query
pub struct FirstRoundBuilder<'a, S: Scalar> {
    commitment_descriptor: Vec<CommittableColumn<'a>>,
    pcs_proof_mles: Vec<Box<dyn MultilinearExtension<S> + 'a>>,
    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
    /// The extra one evaluation lengths used in the proof.
    one_evaluation_lengths: Vec<usize>,
    // The range_length used in sumcheck which is max of all possible ones.
    range_length: usize,
}

impl<'a, S: Scalar> FirstRoundBuilder<'a, S> {
    pub fn new(initial_range_length: usize) -> Self {
        Self {
            commitment_descriptor: Vec::new(),
            pcs_proof_mles: Vec::new(),
            num_post_result_challenges: 0,
            one_evaluation_lengths: Vec::new(),
            range_length: initial_range_length,
        }
    }

    /// Get the range length used in the proof.
    pub(crate) fn range_length(&self) -> usize {
        self.range_length
    }

    /// Update the range length used in the proof only if the new range is larger than the existing range.
    pub(crate) fn update_range_length(&mut self, new_range_length: usize) {
        if new_range_length > self.range_length {
            self.range_length = new_range_length;
        }
    }

    pub fn pcs_proof_mles(&self) -> &[Box<dyn MultilinearExtension<S> + 'a>] {
        &self.pcs_proof_mles
    }

    /// Get the one evaluation lengths used in the proof.
    pub(crate) fn one_evaluation_lengths(&self) -> &[usize] {
        &self.one_evaluation_lengths
    }

    /// Append the length to the list of one evaluation lengths.
    pub(crate) fn produce_one_evaluation_length(&mut self, length: usize) {
        self.update_range_length(length);
        self.one_evaluation_lengths.push(length);
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
        self.pcs_proof_mles.push(Box::new(data));
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    #[tracing::instrument(
        name = "FirstRoundBuilder::commit_intermediate_mles",
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

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "FirstRoundBuilder::evaluate_pcs_proof_mles",
        level = "debug",
        skip_all
    )]
    pub fn evaluate_pcs_proof_mles(&self, evaluation_vec: &[S]) -> Vec<S> {
        log::log_memory_usage("Start");

        let mut res = Vec::with_capacity(self.pcs_proof_mles.len());
        for evaluator in &self.pcs_proof_mles {
            res.push(evaluator.inner_product(evaluation_vec));
        }

        log::log_memory_usage("End");

        res
    }

    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub(super) fn num_post_result_challenges(&self) -> usize {
        self.num_post_result_challenges
    }

    /// Request `cnt` more post result challenges.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub fn request_post_result_challenges(&mut self, cnt: usize) {
        self.num_post_result_challenges += cnt;
    }
}
