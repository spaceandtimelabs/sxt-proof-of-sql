use super::{
    compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
    SumcheckRandomScalars,
};
use crate::base::polynomial::ArkScalar;
use crate::base::polynomial::Scalar;
use crate::base::scalar::ToArkScalar;

/// Evaluations for different MLEs at the random point chosen for sumcheck
pub struct SumcheckMleEvaluations<'a> {
    /// The evaluation of an MLE {x_i} where
    ///     x_i = 1, if i < table_length;
    ///         = 0, otherwise
    #[cfg_attr(not(test), deprecated = "use `get_one_evaluation_ark()` instead")]
    pub one_evaluation: Scalar,

    /// The evaluation of the MLE formed from entrywise random scalars.
    ///
    /// This is used within sumcheck to establish that a given expression
    /// is zero across all entries.
    #[cfg_attr(not(test), deprecated = "use `get_random_evaluation_ark()` instead")]
    pub random_evaluation: Scalar,
    pub pre_result_evaluations: &'a [Scalar],
    pub result_evaluations: &'a [Scalar],
}

impl<'a> SumcheckMleEvaluations<'a> {
    #[tracing::instrument(
        name = "proofs.sql.proof.sumcheck_mle_evaluations.new",
        level = "info",
        skip_all
    )]
    pub fn get_one_evaluation_ark(&self) -> ArkScalar {
        #[allow(deprecated)]
        ToArkScalar::to_ark_scalar(&self.one_evaluation)
    }
    pub fn get_random_evaluation_ark(&self) -> ArkScalar {
        #[allow(deprecated)]
        ToArkScalar::to_ark_scalar(&self.random_evaluation)
    }
    pub fn new(
        table_length: usize,
        evaluation_point: &[Scalar],
        sumcheck_random_scalars: &SumcheckRandomScalars,
        pre_result_evaluations: &'a [Scalar],
        result_evaluations: &'a [Scalar],
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

        #[allow(deprecated)]
        Self {
            one_evaluation,
            random_evaluation,
            pre_result_evaluations,
            result_evaluations,
        }
    }
}
