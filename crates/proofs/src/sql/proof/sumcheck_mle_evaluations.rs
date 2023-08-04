use super::{
    compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
    SumcheckRandomScalars,
};
use crate::base::scalar::ArkScalar;

/// Evaluations for different MLEs at the random point chosen for sumcheck
pub struct SumcheckMleEvaluations<'a> {
    pub table_length: usize,
    pub num_sumcheck_variables: usize,
    /// The evaluation of an MLE {x_i} where
    ///     x_i = 1, if i < table_length;
    ///         = 0, otherwise
    pub one_evaluation: ArkScalar,

    /// The evaluation of the MLE formed from entrywise random scalars.
    ///
    /// This is used within sumcheck to establish that a given expression
    /// is zero across all entries.
    pub random_evaluation: ArkScalar,
    pub pre_result_evaluations: &'a [ArkScalar],
    pub result_evaluations: &'a [ArkScalar],
}

impl<'a> SumcheckMleEvaluations<'a> {
    #[tracing::instrument(
        name = "proofs.sql.proof.sumcheck_mle_evaluations.new",
        level = "info",
        skip_all
    )]
    pub fn new(
        table_length: usize,
        evaluation_point: &[ArkScalar],
        sumcheck_random_scalars: &SumcheckRandomScalars,
        pre_result_evaluations: &'a [ArkScalar],
        result_evaluations: &'a [ArkScalar],
    ) -> Self {
        assert_eq!(
            evaluation_point.len(),
            sumcheck_random_scalars.entrywise_point.len()
        );
        assert_eq!(table_length, sumcheck_random_scalars.table_length);
        let random_evaluation = compute_truncated_lagrange_basis_inner_product(
            table_length,
            evaluation_point,
            sumcheck_random_scalars.entrywise_point,
        );
        let one_evaluation = compute_truncated_lagrange_basis_sum(table_length, evaluation_point);

        Self {
            table_length,
            num_sumcheck_variables: evaluation_point.len(),
            one_evaluation,
            random_evaluation,
            pre_result_evaluations,
            result_evaluations,
        }
    }
}
