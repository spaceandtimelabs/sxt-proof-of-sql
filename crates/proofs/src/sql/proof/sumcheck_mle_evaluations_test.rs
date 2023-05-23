use crate::sql::proof::{ProofCounts, SumcheckRandomScalars};

use super::SumcheckMleEvaluations;

use crate::base::polynomial::Scalar;

#[test]
fn we_can_track_the_evaluation_of_mles_used_within_sumcheck() {
    let counts = ProofCounts {
        table_length: 3,
        offset_generators: 0,
        sumcheck_variables: 2,
        sumcheck_max_multiplicands: 0,
        result_columns: 0,
        anchored_mles: 0,
        intermediate_mles: 0,
        sumcheck_subpolynomials: 0,
    };

    let evaluation_point = [Scalar::from(3u64), Scalar::from(5u64)];
    let random_scalars = [Scalar::from(123u64), Scalar::from(456u64)];

    let sumcheck_random_scalars = SumcheckRandomScalars::new(&counts, &random_scalars);

    let pre_result_evaluations = [Scalar::from(42u64)];
    let result_evaluations = [Scalar::from(51u64)];
    let evals = SumcheckMleEvaluations::new(
        3,
        &evaluation_point,
        &sumcheck_random_scalars,
        &pre_result_evaluations,
        &result_evaluations,
    );
    let expected_eval = (Scalar::one() - evaluation_point[0])
        * (Scalar::one() - evaluation_point[1])
        * (Scalar::one() - random_scalars[0])
        * (Scalar::one() - random_scalars[1])
        + (evaluation_point[0])
            * (Scalar::one() - evaluation_point[1])
            * (random_scalars[0])
            * (Scalar::one() - random_scalars[1])
        + (Scalar::one() - evaluation_point[0])
            * (evaluation_point[1])
            * (Scalar::one() - random_scalars[0])
            * (random_scalars[1]);
    assert_eq!(evals.random_evaluation, expected_eval);

    let expected_eval = (Scalar::one() - evaluation_point[0])
        * (Scalar::one() - evaluation_point[1])
        + (evaluation_point[0]) * (Scalar::one() - evaluation_point[1])
        + (Scalar::one() - evaluation_point[0]) * (evaluation_point[1]);
    assert_eq!(evals.one_evaluation, expected_eval);
}
