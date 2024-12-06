use super::{ProofPlan, VerifiableQueryResult};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    database::{owned_table_utility::*, OwnedColumn, OwnedTable, TableRef, TestAccessor},
    scalar::{Curve25519Scalar, Scalar},
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
    res.clone()
        .verify(expr, accessor, &())
        .expect("Verification failed");

    let (result, proof) = match (&res.result, &res.proof) {
        (Some(result), Some(proof)) => (result, proof),
        (None, None) => return,
        _ => panic!("verification did not catch a proof/result mismatch"),
    };

    // try changing the result
    let mut res_p = res.clone();
    res_p.result = Some(tampered_table(result));
    assert!(res_p.verify(expr, accessor, &()).is_err());

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

fn tampered_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    if table.num_columns() == 0 {
        owned_table([bigint("col", [0; 0])])
    } else if table.num_rows() == 0 {
        append_single_row_to_table(table)
    } else {
        tamper_first_element_of_table(table)
    }
}
fn append_single_row_to_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    OwnedTable::try_from_iter(
        table
            .inner_table()
            .iter()
            .map(|(name, col)| (*name, append_single_row_to_column(col))),
    )
    .expect("Failed to create table")
}
fn append_single_row_to_column<S: Scalar>(column: &OwnedColumn<S>) -> OwnedColumn<S> {
    let mut column = column.clone();
    match &mut column {
        OwnedColumn::Boolean(col) => col.push(false),
        OwnedColumn::TinyInt(col) => col.push(0),
        OwnedColumn::SmallInt(col) => col.push(0),
        OwnedColumn::Int(col) => col.push(0),
        OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.push(0),
        OwnedColumn::VarChar(col) => col.push(String::new()),
        OwnedColumn::Int128(col) => col.push(0),
        OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.push(S::ZERO),
    }
    column
}
fn tamper_first_element_of_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    OwnedTable::try_from_iter(
        table
            .inner_table()
            .iter()
            .enumerate()
            .map(|(i, (name, col))| {
                (
                    *name,
                    if i == 0 {
                        tamper_first_row_of_column(col)
                    } else {
                        col.clone()
                    },
                )
            }),
    )
    .expect("Failed to create table")
}
pub fn tamper_first_row_of_column<S: Scalar>(column: &OwnedColumn<S>) -> OwnedColumn<S> {
    let mut column = column.clone();
    match &mut column {
        OwnedColumn::Boolean(col) => col[0] ^= true,
        OwnedColumn::TinyInt(col) => col[0] += 1,
        OwnedColumn::SmallInt(col) => col[0] += 1,
        OwnedColumn::Int(col) => col[0] += 1,
        OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col[0] += 1,
        OwnedColumn::VarChar(col) => col[0].push('1'),
        OwnedColumn::Int128(col) => col[0] += 1,
        OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col[0] += S::ONE,
    }
    column
}
