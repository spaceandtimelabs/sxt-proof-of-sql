use curve25519_dalek::scalar::Scalar;

/// Evaluations for different MLEs at the random point chosen for sumcheck
pub struct SumcheckMleEvaluations<'a> {
    pub one_evaluation: Scalar,
    pub random_evaluation: Scalar,
    pub pre_result_evaluations: &'a [Scalar],
    pub result_evaluations: &'a [Scalar],
}
