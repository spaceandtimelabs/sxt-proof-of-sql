use super::{FinalRoundBuilder, ProvableQueryResult, SumcheckRandomScalars};
use crate::{
    base::{
        commitment::{Commitment, CommittableColumn},
        database::{Column, ColumnField, ColumnType},
        polynomial::{compute_evaluation_vector, CompositePolynomial, MultilinearExtension},
        scalar::Curve25519Scalar,
    },
    sql::proof::SumcheckSubpolynomialType,
};
use curve25519_dalek::RistrettoPoint;
use num_traits::{One, Zero};

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 0_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        [RistrettoPoint::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )[0]]
    );
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 123_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        [RistrettoPoint::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )[0]]
    );
}

#[test]
fn we_can_evaluate_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
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
fn we_can_form_an_aggregated_sumcheck_polynomial() {
    let mle1 = [1, 2, -1];
    let mle2 = [10i64, 20, 100, 30];
    let mle3 = [2000i64, 3000, 5000, 7000];
    let mut builder = FinalRoundBuilder::new(4, 2, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    builder.produce_intermediate_mle(&mle3[..]);

    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(-Curve25519Scalar::one(), vec![Box::new(&mle1)])],
    );
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(-Curve25519Scalar::from(10u64), vec![Box::new(&mle2)])],
    );
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![(Curve25519Scalar::from(9876u64), vec![Box::new(&mle3)])],
    );

    let multipliers = [
        Curve25519Scalar::from(5u64),
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(50u64),
        Curve25519Scalar::from(25u64),
        Curve25519Scalar::from(11u64),
    ];

    let mut evaluation_vector = vec![Zero::zero(); 4];
    compute_evaluation_vector(&mut evaluation_vector, &multipliers[..2]);

    let poly = builder.make_sumcheck_polynomial(&SumcheckRandomScalars::new(&multipliers, 4, 2));
    let mut expected_poly = CompositePolynomial::new(2);
    let fr = (&evaluation_vector).to_sumcheck_term(2);
    expected_poly.add_product(
        [fr.clone(), (&mle1).to_sumcheck_term(2)],
        -Curve25519Scalar::from(1u64) * multipliers[2],
    );
    expected_poly.add_product(
        [fr, (&mle2).to_sumcheck_term(2)],
        -Curve25519Scalar::from(10u64) * multipliers[3],
    );
    expected_poly.add_product(
        [(&mle3).to_sumcheck_term(2)],
        Curve25519Scalar::from(9876u64) * multipliers[4],
    );
    let random_point = [
        Curve25519Scalar::from(123u64),
        Curve25519Scalar::from(101_112_u64),
    ];
    let eval = poly.evaluate(&random_point);
    let expected_eval = expected_poly.evaluate(&random_point);
    assert_eq!(eval, expected_eval);
}

#[test]
fn we_can_fold_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::new(2, 1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let multipliers = [Curve25519Scalar::from(100u64), Curve25519Scalar::from(2u64)];
    let z = builder.fold_pcs_proof_mles(&multipliers);
    let expected_z = [
        Curve25519Scalar::from(120u64),
        Curve25519Scalar::from(240u64),
    ];
    assert_eq!(z, expected_z);
}

#[test]
fn we_can_consume_post_result_challenges_in_proof_builder() {
    let mut builder = FinalRoundBuilder::new(
        0,
        0,
        vec![
            Curve25519Scalar::from(123),
            Curve25519Scalar::from(456),
            Curve25519Scalar::from(789),
        ],
    );
    assert_eq!(
        Curve25519Scalar::from(789),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(123),
        builder.consume_post_result_challenge()
    );
}
