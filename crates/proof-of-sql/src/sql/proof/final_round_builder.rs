use super::{SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommittableColumn, VecCommitmentExt},
        database::{Column, NullableColumn},
        polynomial::MultilinearExtension,
        scalar::Scalar,
    },
    utils::log,
};
use alloc::{boxed::Box, collections::VecDeque, vec, vec::Vec};
use bumpalo::Bump;

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
    post_result_challenges: VecDeque<S>,
}

impl<'a, S: Scalar> FinalRoundBuilder<'a, S> {
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

    /// Records an IS NULL check for a nullable column
    pub fn record_is_null_check(&mut self, column: &NullableColumn<'a, S>, alloc: &'a Bump) {
        if let Some(presence) = &column.presence {
            // When presence is Some, negate the presence array since presence[i]=true means NOT NULL
            // and we want to record where values ARE NULL (i.e., where presence[i]=false)
            let is_null = alloc.alloc_slice_fill_with(presence.len(), |i| !presence[i]);
            let is_null_column = Column::Boolean(is_null);
            self.produce_intermediate_mle(is_null_column);
        } else {
            // When presence is None, create a constant false MLE since no values are null
            let table_size = column.values.len();
            let all_false = alloc.alloc_slice_fill_copy(table_size, false);
            let constant_false = Column::Boolean(all_false);
            self.produce_intermediate_mle(constant_false);
        }
    }

    /// Records an IS NOT NULL check for a nullable column
    pub fn record_is_not_null_check(&mut self, column: &NullableColumn<'a, S>, alloc: &'a Bump) {
        if let Some(presence) = &column.presence {
            // When presence is Some, use the presence array directly since presence[i]=true means NOT NULL
            let presence_column = Column::Boolean(presence);
            self.produce_intermediate_mle(presence_column);
        } else {
            // When presence is None, all values are non-null so return constant true
            let table_size = column.values.len();
            let all_true = alloc.alloc_slice_fill_copy(table_size, true);
            let constant_true = Column::Boolean(all_true);
            self.produce_intermediate_mle(constant_true);
        }
    }

    /// Records an IS TRUE check for a nullable column
    ///
    /// # Arguments
    /// * `column` - The nullable column to check
    /// * `alloc` - The allocator to use for temporary storage
    /// * `is_or_expr` - Whether this is from an OR expression, which requires special NULL handling
    ///
    /// # Panics
    /// Panics if the provided column is not a boolean column (i.e., if `column.values` is not `Column::Boolean`)
    pub fn record_is_true_check(
        &mut self,
        column: &NullableColumn<'a, S>,
        alloc: &'a Bump,
        is_or_expr: bool,
    ) {
        // Verify that we're working with a boolean column
        if let Column::Boolean(values) = column.values {
            // For IS TRUE, we need to check if the value is both not null and true
            if let Some(presence) = &column.presence {
                // Create a new array that is true only when:
                // 1. The value is not null (presence[i] = true) AND the value is true (values[i] = true)
                // OR (for OR expressions only)
                // 2. Special handling for OR: The value is true (values[i] = true)
                // This implements TRUE OR NULL = TRUE in SQL's three-valued logic
                let mut is_true = Vec::with_capacity(values.len());
                for i in 0..values.len() {
                    if is_or_expr && values[i] {
                        // For OR expressions, if the value is TRUE, keep it TRUE regardless of NULL status
                        is_true.push(true);
                    } else {
                        // For all other cases, apply standard IS TRUE check
                        is_true.push(presence[i] && values[i]);
                    }
                }
                // Use the allocator to ensure the vector lives for the required 'a lifetime
                let is_true_slice = alloc.alloc_slice_copy(&is_true);
                let is_true_column = Column::Boolean(is_true_slice);
                self.produce_intermediate_mle(is_true_column);

                // Create the sumcheck subpolynomial that verifies the IS TRUE constraint
                let mismatch = alloc.alloc_slice_fill_with(values.len(), |i| {
                    let expected = if is_or_expr && values[i] {
                        true
                    } else {
                        presence[i] && values[i]
                    };
                    is_true[i] != expected
                });
                self.produce_sumcheck_subpolynomial(
                    SumcheckSubpolynomialType::Identity,
                    vec![(S::one(), vec![Box::new(&*mismatch)])],
                );
            } else {
                // When presence is None, all values are non-null
                // So we just need to check if the values are true
                self.produce_intermediate_mle(Column::Boolean(values));

                // Create the sumcheck subpolynomial for the no-nulls case
                let mismatch = alloc.alloc_slice_fill_with(values.len(), |i| {
                    let expected = values[i]; // When no nulls, result should match values
                    values[i] != expected
                });
                self.produce_sumcheck_subpolynomial(
                    SumcheckSubpolynomialType::Identity,
                    vec![(S::one(), vec![Box::new(&*mismatch)])],
                );
            }
        } else {
            // IS TRUE can only be applied to boolean expressions
            panic!("IS TRUE can only be applied to boolean expressions");
        }
    }
}
