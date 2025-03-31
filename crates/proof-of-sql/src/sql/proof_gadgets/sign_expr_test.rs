use super::{final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign};
use crate::{
    base::{
        bit::BitDistribution,
        polynomial::MultilinearExtension,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::proof::{
        FinalRoundBuilder, SumcheckMleEvaluations, SumcheckRandomScalars, VerificationBuilderImpl,
    },
};
use alloc::collections::VecDeque;
use bumpalo::Bump;

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_constant_column() {
    let data = [123_i64, 123, 123];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let data: Vec<TestScalar> = data.into_iter().map(TestScalar::from).collect();
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let sign = final_round_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [false; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_negative_constant_column() {
    let data = [-123_i64, -123, -123];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let data: Vec<TestScalar> = data.into_iter().map(TestScalar::from).collect();
    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let sign = final_round_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [true; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn we_can_verify_a_constant_decomposition() {
    let data = [123_i64, 123, 123];

    let dists = [BitDistribution::new::<TestScalar, _>(&data)];
    let scalars = [TestScalar::from(97), TestScalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [TestScalar::from(324), TestScalar::from(97)];
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

    let mut builder = VerificationBuilderImpl::new(
        sumcheck_evaluations,
        &dists,
        &[],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        3,
    );
    let data_eval = (&data).evaluate_at_point(&evaluation_point);
    let eval = verifier_evaluate_sign(&mut builder, data_eval, *chi_eval, Some(8)).unwrap();
    assert_eq!(eval, TestScalar::ZERO);
}

#[test]
fn verification_of_constant_data_fails_if_the_commitment_doesnt_match_the_bit_distribution() {
    let data = [123_i64, 123, 123];

    let dists = [BitDistribution::new::<TestScalar, _>(&data)];
    let scalars = [TestScalar::from(97), TestScalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [TestScalar::from(324), TestScalar::from(97)];
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

    let mut builder = VerificationBuilderImpl::new(
        sumcheck_evaluations,
        &dists,
        &[],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        3,
    );
    let data_eval = TestScalar::from(2) * (&data).evaluate_at_point(&evaluation_point);
    assert!(verifier_evaluate_sign(&mut builder, data_eval, *chi_eval, None).is_err());
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_first_round_evaluate_sign_for_a_constant() {
    let data: &[TestScalar] = &[(-123).into(), (-123).into()];
    let alloc = Bump::new();
    let res = first_round_evaluate_sign(2, &alloc, data);
    let expected_res = [true, true];
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_first_round_evaluate_sign_with_varying_bits_and_fixed_sign(
) {
    let data: &[TestScalar] = &[123.into(), 452.into(), 0.into(), 789.into(), 910.into()];
    let alloc = Bump::new();
    let res = first_round_evaluate_sign(5, &alloc, data);
    let expected_res = [false, false, false, false, false];
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compute_the_correct_sign_of_scalars_using_first_round_evaluate_sign_with_varying_bits_and_sign(
) {
    let data: &[TestScalar] = &[
        123.into(),
        (-452).into(),
        0.into(),
        789.into(),
        (-910).into(),
    ];
    let alloc = Bump::new();
    let res = first_round_evaluate_sign(5, &alloc, data);
    let expected_res = [false, true, false, false, true];
    assert_eq!(res, expected_res);
}
