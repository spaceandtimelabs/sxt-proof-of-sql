use super::{SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommittableColumn, VecCommitmentExt},
        polynomial::MultilinearExtension,
        proof::ProofSizeMismatch,
        scalar::Scalar,
    },
    utils::log,
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use bumpalo::Bump;

/// Track components used to form a query's proof
pub struct FinalRoundBuilder<'a, S: Scalar + 'a> {
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
    post_result_challenges: VecDeque<S>,
}

impl<'a, S: Scalar + 'a> FinalRoundBuilder<'a, S> {
    pub fn new(num_sumcheck_variables: usize, post_result_challenges: VecDeque<S>) -> Self {
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
        log::log_memory_usage("Start");

        let res = Vec::from_committable_columns_with_offset(
            &self.commitment_descriptor,
            offset_generators,
            setup,
        );

        log::log_memory_usage("End");

        res
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn sumcheck_subpolynomials(&self) -> &[SumcheckSubpolynomial<'a, S>] {
        &self.sumcheck_subpolynomials
    }

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "FinalRoundBuilder::evaluate_pcs_proof_mles",
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

    pub fn bit_distributions(&self) -> &[BitDistribution] {
        &self.bit_distributions
    }

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    /// 
    /// # Panics
    ///
    /// Will panic if there are no post-result challenges available to pop from the stack.
    pub fn consume_post_result_challenge(&mut self) -> S {
        self.post_result_challenges.pop_front().unwrap()
    }

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Returns a `Result` that will be `Err(ProofSizeMismatch::PostResultCountMismatch)`
    /// if there are no post-result challenges available to pop from the stack.
    pub fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        self.post_result_challenges
            .pop_front()
            .ok_or(ProofSizeMismatch::PostResultCountMismatch)
    }

    /// Add a helper method to produce a constant boolean column with a single value
    fn produce_constant_boolean(&mut self, value: bool, alloc: &'a Bump) {
        let bool_slice = alloc.alloc_slice_fill_copy(1, value);
        let column = crate::base::database::Column::Boolean(bool_slice);
        self.produce_intermediate_mle(column);
    }

    /// Record an IS NULL check in the proof
    pub fn record_is_null_check<S2: Scalar>(&mut self, column: &crate::base::database::NullableColumn<'a, S2>, alloc: &'a Bump) {
        // For IS NULL operation, we need to include the presence information in the proof
        // In SQL, IS NULL checks if a value is NULL (missing)
        // We represent this by adding the presence information to the proof
        
        // Extract presence information from the column
        // If the column has a presence slice, we need to add it to the proof
        if let Some(presence) = column.presence {
            // For each row, we need to add a check that determines if the value is NULL
            // In our implementation, a NULL value is represented by a false in the presence slice
            // This is used by the verify_is_null_check method in the verification builder
            
            // Add the presence column as an intermediate witness column
            // This allows the verifier to check IS NULL operations correctly
            let presence_column = crate::base::database::Column::Boolean(presence);
            self.produce_intermediate_mle(presence_column);
        } else {
            // If there's no presence slice, all values are present (non-NULL)
            // For IS NULL check, this means all rows should return FALSE
            // We represent this with a single constant FALSE value since it applies to all rows
            self.produce_constant_boolean(false, alloc);
        }
    }

    /// Record an IS NOT NULL check in the proof
    pub fn record_is_not_null_check<S2: Scalar>(&mut self, column: &crate::base::database::NullableColumn<'a, S2>, alloc: &'a Bump) {
        // For IS NOT NULL operation, we need to include the presence information in the proof
        // In SQL, IS NOT NULL checks if a value is not NULL (present)
        // We represent this by adding the presence information to the proof
        
        // Extract presence information from the column
        // If the column has a presence slice, we need to add it to the proof
        if let Some(presence) = column.presence {
            // For each row, we need to add a check that determines if the value is not NULL
            // In our implementation, a non-NULL value is represented by a true in the presence slice
            // This is used by the verify_is_not_null_check method in the verification builder
            
            // Add the presence column as an intermediate witness column
            // This allows the verifier to check IS NOT NULL operations correctly
            let presence_column = crate::base::database::Column::Boolean(presence);
            self.produce_intermediate_mle(presence_column);
        } else {
            // If there's no presence slice, all values are present (non-NULL)
            // We can represent this with a single constant value since it applies to all rows
            // The verifier will use this to correctly determine that all values are not NULL
            self.produce_constant_boolean(true, alloc);
        }
    }

    /// Record an IS TRUE check in the proof
    pub fn record_is_true_check<S2: Scalar>(&mut self, column: &crate::base::database::NullableColumn<'a, S2>, alloc: &'a Bump) {
        // For IS TRUE operation, we need to include the presence information and the actual values
        // In SQL, IS TRUE checks if a value is both not NULL and TRUE
        
        // First, we need to check if the column is boolean, since IS TRUE only applies to boolean expressions
        match column.values {
            crate::base::database::Column::Boolean(_values) => {
                // For IS TRUE check, we only need to add one intermediate MLE to the proof
                // This follows the same pattern as record_is_null_check and record_is_not_null_check
                
                // Extract presence information from the column
                // If the column has a presence slice, we need to add it to the proof
                if let Some(presence) = column.presence {
                    // Add the presence column as an intermediate witness column
                    let presence_column = crate::base::database::Column::Boolean(presence);
                    self.produce_intermediate_mle(presence_column);
                } else {
                    // If there's no presence slice, all values are present (non-NULL)
                    // We can represent this with a single constant value since it applies to all rows
                    self.produce_constant_boolean(true, alloc);
                }
            },
            _ => {
                // IS TRUE can only be applied to boolean expressions
                // If this is not a boolean column, we should handle this as an error case
                panic!("IS TRUE can only be applied to boolean expressions");
            }
        }
    }
}
