use curve25519_dalek::scalar::Scalar;

/// Evaluations for different MLEs at the random point chosen for sumcheck
pub struct SumcheckMleEvaluations<'a> {
    /// The evaluation of an MLE {x_i} where
    ///     x_i = 1, if i < table_length;
    ///         = 0, otherwise
    pub one_evaluation: Scalar,

    /// The evaluation of the MLE formed from entrywise random scalars.
    ///
    /// This is used within sumcheck to establish that a given expression
    /// is zero across all entries.
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
    pub fn new(
        table_length: usize,
        evaluation_vec: &[Scalar],
        entrywise_random_scalars: &[Scalar],
        pre_result_evaluations: &'a [Scalar],
        result_evaluations: &'a [Scalar],
    ) -> Self {
        assert_eq!(evaluation_vec.len(), entrywise_random_scalars.len());
        assert_eq!(table_length, evaluation_vec.len());
        let mut random_evaluation = Scalar::zero();
        for (ei, ri) in evaluation_vec.iter().zip(entrywise_random_scalars.iter()) {
            random_evaluation += ei * ri;
        }
        let mut one_evaluation = Scalar::zero();
        for ei in evaluation_vec.iter().take(table_length) {
            one_evaluation += ei;
        }
        Self {
            one_evaluation,
            random_evaluation,
            pre_result_evaluations,
            result_evaluations,
        }
    }
}
