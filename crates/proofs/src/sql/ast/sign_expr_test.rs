use super::{count_sign, prover_evaluate_sign, verifier_evaluate_sign};
use crate::{
    base::{
        bit::BitDistribution,
        database::{RecordBatchTestAccessor, TestAccessor},
        scalar::{compute_commitment_for_testing, ArkScalar},
    },
    record_batch,
    sql::proof::{
        CountBuilder, ProofBuilder, SumcheckMleEvaluations, SumcheckRandomScalars,
        VerificationBuilder,
    },
};
use blitzar::compute::get_one_commit;
use bumpalo::Bump;
use num_traits::Zero;

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_constant_column() {
    let data = [123_i64, 123, 123];
    let dist = BitDistribution::new(&data);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, record_batch!("a" => data), 0);

    let alloc = Bump::new();
    let data: Vec<ArkScalar> = data.into_iter().map(ArkScalar::from).collect();
    let mut builder = ProofBuilder::new(3, 2);
    let sign = prover_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [false; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn prover_evaluation_generates_the_bit_distribution_of_a_negative_constant_column() {
    let data = [-123_i64, -123, -123];
    let dist = BitDistribution::new(&data);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, record_batch!("a" => data), 0);

    let alloc = Bump::new();
    let data: Vec<ArkScalar> = data.into_iter().map(ArkScalar::from).collect();
    let mut builder = ProofBuilder::new(3, 2);
    let sign = prover_evaluate_sign(&mut builder, &alloc, &data);
    assert_eq!(sign, [true; 3]);
    assert_eq!(builder.bit_distributions(), [dist]);
}

#[test]
fn count_fails_if_a_bit_distribution_is_out_of_range() {
    let dists = [BitDistribution::new(&[
        ArkScalar::from(3) * ArkScalar::from(u128::MAX)
    ])];
    let mut builder = CountBuilder::new(&dists);
    assert!(count_sign(&mut builder).is_err());
}

#[test]
fn count_fails_if_no_bit_distribution_is_available() {
    let mut builder = CountBuilder::new(&[]);
    assert!(count_sign(&mut builder).is_err());
}

#[test]
fn we_can_verify_a_constant_decomposition() {
    let data = [123_i64, 123, 123];
    let one_commit = get_one_commit(data.len() as u64);

    let dists = [BitDistribution::new(&data)];
    let scalars = [ArkScalar::from(97), ArkScalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [ArkScalar::from(324), ArkScalar::from(97)];
    let sumcheck_evaluations = SumcheckMleEvaluations::new(
        data.len(),
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    );

    let mut builder = VerificationBuilder::new(0, sumcheck_evaluations, &dists, &[], &[], &[]);
    let commit = compute_commitment_for_testing(&data, 0);
    let eval = verifier_evaluate_sign(&mut builder, &commit, &one_commit).unwrap();
    assert_eq!(eval, ArkScalar::zero());
}

#[test]
fn verification_of_constant_data_fails_if_the_commitment_doesnt_match_the_bit_distribution() {
    let data = [123_i64, 123, 123];
    let one_commit = get_one_commit(data.len() as u64);

    let dists = [BitDistribution::new(&data)];
    let scalars = [ArkScalar::from(97), ArkScalar::from(3432)];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, data.len(), 2);
    let evaluation_point = [ArkScalar::from(324), ArkScalar::from(97)];
    let sumcheck_evaluations = SumcheckMleEvaluations::new(
        data.len(),
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    );

    let mut builder = VerificationBuilder::new(0, sumcheck_evaluations, &dists, &[], &[], &[]);
    let commit = ArkScalar::from(2) * compute_commitment_for_testing(&data, 0);
    assert!(verifier_evaluate_sign(&mut builder, &commit, &one_commit).is_err());
}
