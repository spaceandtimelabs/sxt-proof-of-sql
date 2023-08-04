use crate::sql::proof::SumcheckRandomScalars;

use super::SumcheckMleEvaluations;

use crate::base::scalar::ArkScalar;
use num_traits::One;

#[test]
fn we_can_track_the_evaluation_of_mles_used_within_sumcheck() {
    let evaluation_point = [ArkScalar::from(3u64), ArkScalar::from(5u64)];
    let random_scalars = [ArkScalar::from(123u64), ArkScalar::from(456u64)];

    let sumcheck_random_scalars = SumcheckRandomScalars::new(&random_scalars, 3, 2);

    let pre_result_evaluations = [ArkScalar::from(42u64)];
    let result_evaluations = [ArkScalar::from(51u64)];
    let evals = SumcheckMleEvaluations::new(
        3,
        &evaluation_point,
        &sumcheck_random_scalars,
        &pre_result_evaluations,
        &result_evaluations,
    );
    let expected_eval = (ArkScalar::one() - evaluation_point[0])
        * (ArkScalar::one() - evaluation_point[1])
        * (ArkScalar::one() - random_scalars[0])
        * (ArkScalar::one() - random_scalars[1])
        + (evaluation_point[0])
            * (ArkScalar::one() - evaluation_point[1])
            * (random_scalars[0])
            * (ArkScalar::one() - random_scalars[1])
        + (ArkScalar::one() - evaluation_point[0])
            * (evaluation_point[1])
            * (ArkScalar::one() - random_scalars[0])
            * (random_scalars[1]);
    assert_eq!(evals.random_evaluation, expected_eval);

    let expected_eval = (ArkScalar::one() - evaluation_point[0])
        * (ArkScalar::one() - evaluation_point[1])
        + (evaluation_point[0]) * (ArkScalar::one() - evaluation_point[1])
        + (ArkScalar::one() - evaluation_point[0]) * (evaluation_point[1]);
    assert_eq!(evals.one_evaluation, expected_eval);
}
