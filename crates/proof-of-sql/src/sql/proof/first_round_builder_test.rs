use super::FirstRoundBuilder;
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    scalar::Curve25519Scalar,
};
use curve25519_dalek::RistrettoPoint;

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(2);
    builder.produce_intermediate_mle(&mle1[..]);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 0_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    let expected_commitments: Vec<RistrettoPoint> = RistrettoPoint::compute_commitments(
        &[
            CommittableColumn::from(&mle1[..]),
            CommittableColumn::from(&mle2[..]),
        ],
        offset_generators,
        &(),
    );
    assert_eq!(commitments, expected_commitments,);
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(2);
    builder.produce_intermediate_mle(&mle1[..]);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 123_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    let expected_commitments: Vec<RistrettoPoint> = RistrettoPoint::compute_commitments(
        &[
            CommittableColumn::from(&mle1[..]),
            CommittableColumn::from(&mle2[..]),
        ],
        offset_generators,
        &(),
    );
    assert_eq!(commitments, expected_commitments,);
}

#[test]
fn we_can_evaluate_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(2);
    builder.produce_intermediate_mle(&mle1[..]);
    builder.produce_intermediate_mle(&mle2[..]);
    let evaluation_vec = [
        Curve25519Scalar::from(100u64),
        Curve25519Scalar::from(10u64),
    ];
    let evals = builder.evaluate_pcs_proof_mles(&evaluation_vec);
    let expected_evals = [
        Curve25519Scalar::from(120u64),
        Curve25519Scalar::from(1200u64),
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_add_post_result_challenges() {
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(0);
    assert_eq!(builder.num_post_result_challenges(), 0);
    builder.request_post_result_challenges(1);
    assert_eq!(builder.num_post_result_challenges(), 1);
    builder.request_post_result_challenges(2);
    assert_eq!(builder.num_post_result_challenges(), 3);
}
