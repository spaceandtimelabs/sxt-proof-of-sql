use super::SumcheckMleEvaluations;
use crate::{base::scalar::Curve25519Scalar, sql::proof::SumcheckRandomScalars};
use num_traits::One;

#[test]
fn we_can_track_the_evaluation_of_mles_used_within_sumcheck() {
    let evaluation_point = [Curve25519Scalar::from(3u64), Curve25519Scalar::from(5u64)];
    let random_scalars = [
        Curve25519Scalar::from(123u64),
        Curve25519Scalar::from(456u64),
    ];

    let sumcheck_random_scalars = SumcheckRandomScalars::new(&random_scalars, 3, 2);

    let pcs_proof_evaluations = [Curve25519Scalar::from(42u64)];
    let evals = SumcheckMleEvaluations::new(
        3,
        &[3, 3],
        &evaluation_point,
        &sumcheck_random_scalars,
        &pcs_proof_evaluations,
    );
    let expected_eval = (Curve25519Scalar::one() - evaluation_point[0])
        * (Curve25519Scalar::one() - evaluation_point[1])
        * (Curve25519Scalar::one() - random_scalars[0])
        * (Curve25519Scalar::one() - random_scalars[1])
        + (evaluation_point[0])
            * (Curve25519Scalar::one() - evaluation_point[1])
            * (random_scalars[0])
            * (Curve25519Scalar::one() - random_scalars[1])
        + (Curve25519Scalar::one() - evaluation_point[0])
            * (evaluation_point[1])
            * (Curve25519Scalar::one() - random_scalars[0])
            * (random_scalars[1]);
    assert_eq!(evals.random_evaluation, expected_eval);

    let expected_eval = (Curve25519Scalar::one() - evaluation_point[0])
        * (Curve25519Scalar::one() - evaluation_point[1])
        + (evaluation_point[0]) * (Curve25519Scalar::one() - evaluation_point[1])
        + (Curve25519Scalar::one() - evaluation_point[0]) * (evaluation_point[1]);
    assert_eq!(
        *evals.one_evaluations.values().next().unwrap(),
        expected_eval
    );
}
