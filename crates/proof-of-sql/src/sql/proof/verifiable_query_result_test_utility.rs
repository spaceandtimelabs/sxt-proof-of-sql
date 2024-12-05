use super::{
    verifiable_query_result_test::EmptyTestQueryExpr, ProofPlan, ProvableQueryResult, QueryProof,
    VerifiableQueryResult,
};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    database::{
        owned_table_utility::*, Column, CommitmentAccessor, OwnedTableTestAccessor, TableRef,
        TestAccessor,
    },
    scalar::Curve25519Scalar,
};
use blitzar::proof::InnerProductProof;
use curve25519_dalek::{ristretto::RistrettoPoint, traits::Identity};
use num_traits::One;
use serde::Serialize;

/// This function takes a valid `verifiable_result`, copies it, tweaks it, and checks that
/// verification fails.
///
/// It's useful as a tool for testing proof code.
///
/// # Panics
///
/// Will panic if:
/// - The verification of `res` does not succeed, causing the assertion `assert!(res.verify(...).is_ok())` to fail.
/// - `res.proof` is `None`, causing `res.proof.as_ref().unwrap()` to panic.
/// - Attempting to modify `pcs_proof_evaluations` or `commitments` if `res_p.proof` is `None`, leading to a panic on `unwrap()`.
/// - `fake_accessor.update_offset` fails, causing a panic if it is designed to do so in the implementation.
pub fn exercise_verification(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl TestAccessor<RistrettoPoint>,
    table_ref: TableRef,
) {
    let verification_result = res.clone().verify(expr, accessor, &());
    assert!(
        verification_result.is_ok(),
        "Verification failed: {:?}",
        verification_result.err()
    );

    // try changing the result
    tamper_result(res, expr, accessor);

    if res.proof.is_none() {
        return;
    }
    let proof = res.proof.as_ref().unwrap();

    // try changing MLE evaluations
    for i in 0..proof.pcs_proof_evaluations.len() {
        let mut res_p = res.clone();
        res_p.proof.as_mut().unwrap().pcs_proof_evaluations[i] += Curve25519Scalar::one();
        assert!(res_p.verify(expr, accessor, &()).is_err());
    }

    // try changing intermediate commitments
    let commit_p = RistrettoPoint::compute_commitments(
        &[CommittableColumn::BigInt(&[
            353_453_245_i64,
            93_402_346_i64,
        ])],
        0_usize,
        &(),
    )[0];

    for i in 0..proof.commitments.len() {
        let mut res_p = res.clone();
        res_p.proof.as_mut().unwrap().commitments[i] = commit_p;
        assert!(res_p.verify(expr, accessor, &()).is_err());
    }

    // try changing the offset
    //
    // Note: in the n = 1 case with proof.commmitments all the identity element,
    // the inner product proof isn't dependent on the generators since it simply sends the input
    // vector; hence, changing the offset would have no effect.
    if accessor.get_length(table_ref) > 1
        || proof.commitments.iter().any(|&c| c != Identity::identity())
    {
        let offset_generators = accessor.get_offset(table_ref);
        let mut fake_accessor = accessor.clone();
        fake_accessor.update_offset(table_ref, offset_generators);
        res.clone().verify(expr, &fake_accessor, &()).unwrap();
        fake_accessor.update_offset(table_ref, offset_generators + 1);
        assert!(res.clone().verify(expr, &fake_accessor, &()).is_err());
    }
}

fn tamper_no_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    // add a result
    let mut res_p = res.clone();
    let cols: [Column<'_, Curve25519Scalar>; 1] = [Column::BigInt(&[0_i64; 0])];
    res_p.provable_result = Some(ProvableQueryResult::new(0, &cols));
    assert!(res_p.verify(expr, accessor, &()).is_err());

    // add a proof
    let mut res_p = res.clone();
    let expr_p = EmptyTestQueryExpr {
        length: 1,
        ..Default::default()
    };
    let column = vec![1_i64; 1];
    let accessor_p = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("bogus_col", column)]),
        0,
        (),
    );
    let (proof, _result) = QueryProof::new(&expr_p, &accessor_p, &());
    res_p.proof = Some(proof);
    assert!(res_p.verify(expr, accessor, &()).is_err());
}

fn tamper_empty_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    // try to add a result
    let mut res_p = res.clone();
    let cols: [Column<'_, Curve25519Scalar>; 1] = [Column::BigInt(&[123_i64])];
    res_p.provable_result = Some(ProvableQueryResult::new(1, &cols));
    assert!(res_p.verify(expr, accessor, &()).is_err());
}

/// # Panics
///
/// Will panic if:
/// - `res.provable_result` is `None`, which leads to calling `unwrap()` on it in the subsequent
///   code and may cause an unexpected behavior.
/// - The assertion `assert!(res_p.verify(expr, accessor, &()).is_err())` fails, indicating that the
///   verification did not fail as expected after tampering.
fn tamper_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    if res.provable_result.is_none() {
        tamper_no_result(res, expr, accessor);
        return;
    }
    let provable_res = res.provable_result.as_ref().unwrap();

    if provable_res.table_length() == 0 {
        tamper_empty_result(res, expr, accessor);
        return;
    }

    // try to change data
    let mut res_p = res.clone();
    let mut provable_res_p = provable_res.clone();
    provable_res_p.data_mut()[0] += 1;
    res_p.provable_result = Some(provable_res_p);
    assert!(res_p.verify(expr, accessor, &()).is_err());
}
