use super::SumcheckMleEvaluations;

use curve25519_dalek::scalar::Scalar;

#[test]
fn we_can_track_the_evaluation_of_mles_used_within_sumcheck() {
    let evaluation_vec = [
        Scalar::from(3u64),
        Scalar::from(5u64),
        -Scalar::from(1u64),
        Scalar::from(10u64),
    ];
    let random_scalars = [
        Scalar::from(123u64),
        Scalar::from(456u64),
        Scalar::from(789u64),
        Scalar::from(101112u64),
    ];
    let pre_result_evaluations = [Scalar::from(42u64)];
    let result_evaluations = [Scalar::from(51u64)];
    let evals = SumcheckMleEvaluations::new(
        3,
        &evaluation_vec,
        &random_scalars,
        &pre_result_evaluations,
        &result_evaluations,
    );
    let expected_eval = evaluation_vec[0] * random_scalars[0]
        + evaluation_vec[1] * random_scalars[1]
        + evaluation_vec[2] * random_scalars[2]
        + evaluation_vec[3] * random_scalars[3];
    assert_eq!(evals.random_evaluation, expected_eval);

    let expected_eval = evaluation_vec[0] + evaluation_vec[1] + evaluation_vec[2];
    assert_eq!(evals.one_evaluation, expected_eval);
}
