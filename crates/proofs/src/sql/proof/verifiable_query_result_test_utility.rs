use super::{
    DenseProvableResultColumn, ProofCounts, ProvableQueryResult, ProvableResultColumn, QueryExpr,
    QueryProof, TestQueryExpr, VerifiableQueryResult,
};

use crate::base::database::{CommitmentAccessor, MetadataAccessor, TableRef, TestAccessor};
use crate::base::scalar::compute_commitment_for_testing;

use curve25519_dalek::scalar::Scalar;

/// This function takes a valid verifiable_result, copies it, tweaks it, and checks that
/// verification fails.
///
/// It's useful as a tool for testing proof code.
pub fn exercise_verification(
    res: &VerifiableQueryResult,
    expr: &dyn QueryExpr,
    accessor: &TestAccessor,
    table_ref: TableRef,
) {
    assert!(res.verify(expr, accessor).is_ok());

    // try changing the result
    tamper_result(res, expr, accessor);

    if res.proof.is_none() {
        return;
    }
    let proof = res.proof.as_ref().unwrap();

    // try changing MLE evaluations
    for i in 0..proof.pre_result_mle_evaluations.len() {
        let mut res_p = res.clone();
        res_p.proof.as_mut().unwrap().pre_result_mle_evaluations[i] += Scalar::one();
        assert!(res_p.verify(expr, accessor).is_err());
    }

    // try changing intermediate commitments
    let commit_p = compute_commitment_for_testing(
        &[353453245u64, 93402346u64][..], // some arbitrary values
        0_usize,
    )
    .compress();
    for i in 0..proof.commitments.len() {
        let mut res_p = res.clone();
        res_p.proof.as_mut().unwrap().commitments[i] = commit_p;
        assert!(res_p.verify(expr, accessor).is_err());
    }

    // try changing the offset
    let offset_generators = accessor.get_offset(table_ref);
    let mut fake_accessor = accessor.clone();
    fake_accessor.update_offset(table_ref, offset_generators);
    res.verify(expr, &fake_accessor).unwrap().unwrap();
    fake_accessor.update_offset(table_ref, offset_generators + 1);
    assert!(res.verify(expr, &fake_accessor).is_err());
}

fn tamper_no_result(
    res: &VerifiableQueryResult,
    expr: &dyn QueryExpr,
    accessor: &impl CommitmentAccessor,
) {
    // add a result
    let mut res_p = res.clone();
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::<i64>::new(&[][..]))];
    res_p.provable_result = Some(ProvableQueryResult::new(&[][..], &cols));
    assert!(res_p.verify(expr, accessor).is_err());

    // add a proof
    let mut res_p = res.clone();
    let counts = ProofCounts {
        table_length: 1,
        sumcheck_variables: 1,
        ..Default::default()
    };
    let expr_p = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor_p = TestAccessor::new();
    let (proof, _result) = QueryProof::new(&expr_p, &accessor_p, &counts);
    res_p.proof = Some(proof);
    assert!(res_p.verify(expr, accessor).is_err());
}

fn tamper_empty_result(
    res: &VerifiableQueryResult,
    expr: &dyn QueryExpr,
    accessor: &impl CommitmentAccessor,
) {
    // try to add a result
    let mut res_p = res.clone();
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::<i64>::new(&[123][..]))];
    res_p.provable_result = Some(ProvableQueryResult::new(&[0][..], &cols));
    assert!(res_p.verify(expr, accessor).is_err());
}

fn tamper_result(
    res: &VerifiableQueryResult,
    expr: &dyn QueryExpr,
    accessor: &impl CommitmentAccessor,
) {
    if res.provable_result.is_none() {
        tamper_no_result(res, expr, accessor);
        return;
    }
    let provable_res = res.provable_result.as_ref().unwrap();
    if provable_res.indexes.is_empty() {
        tamper_empty_result(res, expr, accessor);
        return;
    }

    // try to change an index
    let mut res_p = res.clone();
    let mut provable_res_p = provable_res.clone();
    provable_res_p.indexes[0] += 1;
    res_p.provable_result = Some(provable_res_p);
    assert!(res_p.verify(expr, accessor).is_err());

    // try to change data
    let mut res_p = res.clone();
    let mut provable_res_p = provable_res.clone();
    provable_res_p.data[0] += 1;
    res_p.provable_result = Some(provable_res_p);
    assert!(res_p.verify(expr, accessor).is_err());
}
