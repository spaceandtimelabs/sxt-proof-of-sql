use super::{SumcheckMleEvaluations, VerificationBuilder};
use crate::base::scalar::ArkScalar;
use curve25519_dalek::{ristretto::RistrettoPoint, traits::Identity};
use num_traits::Zero;
use rand_core::OsRng;

#[test]
fn an_empty_sumcheck_polynomial_evaluates_to_zero() {
    let mle_evaluations = SumcheckMleEvaluations {
        table_length: 1,
        num_sumcheck_variables: 1,
        ..Default::default()
    };
    let builder = VerificationBuilder::<RistrettoPoint>::new(
        0,
        mle_evaluations,
        &[][..],
        &[][..],
        &[][..],
        &[][..],
        Vec::new(),
    );
    assert_eq!(builder.sumcheck_evaluation(), ArkScalar::zero());
    assert_eq!(
        builder.folded_pre_result_commitment(),
        RistrettoPoint::identity()
    );
}

#[test]
fn we_build_up_a_sumcheck_polynomial_evaluation_from_subpolynomial_evaluations() {
    let mle_evaluations = SumcheckMleEvaluations {
        table_length: 1,
        num_sumcheck_variables: 1,
        ..Default::default()
    };
    let subpolynomial_multipliers = [ArkScalar::from(10u64), ArkScalar::from(100u64)];
    let mut builder = VerificationBuilder::<RistrettoPoint>::new(
        0,
        mle_evaluations,
        &[][..],
        &[][..],
        &subpolynomial_multipliers,
        &[][..],
        Vec::new(),
    );
    builder.produce_sumcheck_subpolynomial_evaluation(&ArkScalar::from(2u64));
    builder.produce_sumcheck_subpolynomial_evaluation(&ArkScalar::from(3u64));
    let expected_sumcheck_evaluation = subpolynomial_multipliers[0] * ArkScalar::from(2u64)
        + subpolynomial_multipliers[1] * ArkScalar::from(3u64);
    assert_eq!(builder.sumcheck_evaluation(), expected_sumcheck_evaluation);
}

#[test]
fn we_build_up_the_folded_pre_result_commitment() {
    let pre_result_evaluations = [ArkScalar::from(123u64), ArkScalar::from(456u64)];
    let mle_evaluations = SumcheckMleEvaluations {
        table_length: 1,
        num_sumcheck_variables: 1,
        pre_result_evaluations: &pre_result_evaluations,
        ..Default::default()
    };
    let mut rng = OsRng;
    let commit1 = RistrettoPoint::random(&mut rng);
    let commit2 = RistrettoPoint::random(&mut rng);
    let intermediate_commitments = [commit2];
    let inner_product_multipliers = [ArkScalar::from(10u64), ArkScalar::from(100u64)];
    let mut builder = VerificationBuilder::new(
        0,
        mle_evaluations,
        &[][..],
        &intermediate_commitments,
        &[][..],
        &inner_product_multipliers,
        Vec::new(),
    );
    let eval = builder.consume_anchored_mle(&commit1);
    assert_eq!(eval, ArkScalar::from(123u64));
    let eval = builder.consume_intermediate_mle();
    assert_eq!(eval, ArkScalar::from(456u64));
    let expected_folded_pre_result_commit =
        inner_product_multipliers[0] * commit1 + inner_product_multipliers[1] * commit2;
    assert_eq!(
        builder.folded_pre_result_commitment(),
        expected_folded_pre_result_commit
    );
    let expected_folded_pre_result_eval = inner_product_multipliers[0] * ArkScalar::from(123u64)
        + inner_product_multipliers[1] * ArkScalar::from(456u64);
    assert_eq!(
        builder.folded_pre_result_evaluation(),
        expected_folded_pre_result_eval
    );
}

#[test]
fn we_can_consume_result_evaluations() {
    let result_evaluations = [ArkScalar::from(123u64), ArkScalar::from(456u64)];
    let mle_evaluations = SumcheckMleEvaluations {
        table_length: 1,
        num_sumcheck_variables: 1,
        result_evaluations: &result_evaluations,
        ..Default::default()
    };
    let mut builder = VerificationBuilder::<RistrettoPoint>::new(
        0,
        mle_evaluations,
        &[][..],
        &[][..],
        &[][..],
        &[][..],
        Vec::new(),
    );
    assert_eq!(builder.consume_result_mle(), ArkScalar::from(123u64));
    assert_eq!(builder.consume_result_mle(), ArkScalar::from(456u64));
}

#[test]
fn we_can_consume_post_result_challenges_in_proof_builder() {
    let mut builder = VerificationBuilder::<RistrettoPoint>::new(
        0,
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        &[][..],
        &[][..],
        vec![
            ArkScalar::from(123),
            ArkScalar::from(456),
            ArkScalar::from(789),
        ],
    );
    assert_eq!(
        ArkScalar::from(789),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        ArkScalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        ArkScalar::from(123),
        builder.consume_post_result_challenge()
    );
}
