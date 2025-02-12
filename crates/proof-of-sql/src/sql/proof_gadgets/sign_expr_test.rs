use super::{prover_evaluate_sign, result_evaluate_sign, verifier_evaluate_sign};
use crate::{
    base::{
        bit::BitDistribution,
        polynomial::MultilinearExtension,
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::proof::{
        FinalRoundBuilder, StandardVerificationBuilder, SumcheckMleEvaluations,
        SumcheckRandomScalars,
    },
};
use alloc::collections::VecDeque;
use bumpalo::Bump;

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_constant_column() {
    let data = [123_i64, 123, 123];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let data: Vec<Curve25519Scalar> = data.into_iter().map(Curve25519Scalar::from).collect();
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let sign = prover_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [false; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_negative_constant_column() {
    let data = [-123_i64, -123, -123];
    let dist = BitDistribution::new::<Curve25519Scalar, _>(&data);
    let alloc = Bump::new();
    let data: Vec<Curve25519Scalar> = data.into_iter().map(Curve25519Scalar::from).collect();
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let sign = prover_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [true; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn we_can_verify_a_constant_decomposition() {
    let data = [123_i64, 123, 123];

    let dists = [BitDistribution::new::<Curve25519Scalar, _>(&data)];
    let scalars = [Curve25519Scalar::from(97), Curve25519Scalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [Curve25519Scalar::from(324), Curve25519Scalar::from(97)];
    let sumcheck_evaluations = SumcheckMleEvaluations::new(
        data.len(),
        [data.len()],
        [],
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    );
    let chi_evals = sumcheck_evaluations.chi_evaluations.clone();
    let chi_eval = chi_evals.values().next().unwrap();

    let mut builder = StandardVerificationBuilder::new(
        sumcheck_evaluations,
        &dists,
        &[],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        3,
    );
    let data_eval = (&data).evaluate_at_point(&evaluation_point);
    let eval = verifier_evaluate_sign(&mut builder, data_eval, *chi_eval).unwrap();
    assert_eq!(eval, Curve25519Scalar::ZERO);
}

#[test]
fn verification_of_constant_data_fails_if_the_commitment_doesnt_match_the_bit_distribution() {
    let data = [123_i64, 123, 123];

    let dists = [BitDistribution::new::<Curve25519Scalar, _>(&data)];
    let scalars = [Curve25519Scalar::from(97), Curve25519Scalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [Curve25519Scalar::from(324), Curve25519Scalar::from(97)];
    let sumcheck_evaluations = SumcheckMleEvaluations::new(
        data.len(),
        [data.len()],
        [],
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    );
    let chi_evals = sumcheck_evaluations.chi_evaluations.clone();
    let chi_eval = chi_evals.values().next().unwrap();

    let mut builder = StandardVerificationBuilder::new(
        sumcheck_evaluations,
        &dists,
        &[],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        3,
    );
    let data_eval = Curve25519Scalar::from(2) * (&data).evaluate_at_point(&evaluation_point);
    assert!(verifier_evaluate_sign(&mut builder, data_eval, *chi_eval).is_err());
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_result_evaluate_sign_for_a_constant() {
    let data: &[Curve25519Scalar] = &[(-123).into(), (-123).into()];
    let alloc = Bump::new();
    let res = result_evaluate_sign(2, &alloc, data);
    let expected_res = [true, true];
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_result_evaluate_sign_with_varying_bits_and_fixed_sign(
) {
    let data: &[Curve25519Scalar] = &[123.into(), 452.into(), 0.into(), 789.into(), 910.into()];
    let alloc = Bump::new();
    let res = result_evaluate_sign(5, &alloc, data);
    let expected_res = [false, false, false, false, false];
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_result_evaluate_sign_with_varying_bits_and_sign(
) {
    let data: &[Curve25519Scalar] = &[
        123.into(),
        (-452).into(),
        0.into(),
        789.into(),
        (-910).into(),
    ];
    let alloc = Bump::new();
    let res = result_evaluate_sign(5, &alloc, data);
    let expected_res = [false, true, false, false, true];
    assert_eq!(res, expected_res);
}
