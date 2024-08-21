use super::{
    verifiable_query_result_test::EmptyTestQueryExpr, ProofExecutionPlan, ProvableQueryResult,
    ProvableResultColumn, QueryProof, VerifiableQueryResult,
};
use crate::{
    base::{
        database::{CommitmentAccessor, OwnedTableTestAccessor, TableRef, TestAccessor},
        scalar::{compute_commitment_for_testing, Curve25519Scalar},
    },
    sql::proof::Indexes,
};
use blitzar::proof::InnerProductProof;
use curve25519_dalek::{ristretto::RistrettoPoint, traits::Identity};
use num_traits::One;
use serde::Serialize;

/// This function takes a valid verifiable_result, copies it, tweaks it, and checks that
/// verification fails.
///
/// It's useful as a tool for testing proof code.
pub fn exercise_verification(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofExecutionPlan<RistrettoPoint> + Serialize),
    accessor: &impl TestAccessor<RistrettoPoint>,
    table_ref: TableRef,
) {
    assert!(res.verify(expr, accessor, &()).is_ok());

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
    let commit_p = compute_commitment_for_testing(
        &[353453245u64, 93402346u64][..], // some arbitrary values
        0_usize,
    );
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
        res.verify(expr, &fake_accessor, &()).unwrap();
        fake_accessor.update_offset(table_ref, offset_generators + 1);
        assert!(res.verify(expr, &fake_accessor, &()).is_err());
    }
}

fn tamper_no_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofExecutionPlan<RistrettoPoint> + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    // add a result
    let mut res_p = res.clone();
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new([0_i64; 0])];
    res_p.provable_result = Some(ProvableQueryResult::new(&Indexes::Sparse(vec![]), &cols));
    assert!(res_p.verify(expr, accessor, &()).is_err());

    // add a proof
    let mut res_p = res.clone();
    let expr_p = EmptyTestQueryExpr {
        length: 1,
        ..Default::default()
    };
    let accessor_p = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let (proof, _result) = QueryProof::new(&expr_p, &accessor_p, &());
    res_p.proof = Some(proof);
    assert!(res_p.verify(expr, accessor, &()).is_err());
}

fn tamper_empty_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofExecutionPlan<RistrettoPoint> + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    // try to add a result
    let mut res_p = res.clone();
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new([123_i64])];
    res_p.provable_result = Some(ProvableQueryResult::new(&Indexes::Sparse(vec![0]), &cols));
    assert!(res_p.verify(expr, accessor, &()).is_err());
}

fn tamper_result(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofExecutionPlan<RistrettoPoint> + Serialize),
    accessor: &impl CommitmentAccessor<RistrettoPoint>,
) {
    if res.provable_result.is_none() {
        tamper_no_result(res, expr, accessor);
        return;
    }
    let provable_res = res.provable_result.as_ref().unwrap();
    if provable_res.indexes().is_empty() {
        tamper_empty_result(res, expr, accessor);
        return;
    }

    // try to change an index
    let mut res_p = res.clone();
    let mut provable_res_p = provable_res.clone();
    match provable_res_p.indexes_mut() {
        Indexes::Sparse(indexes) => indexes[0] += 1,
        Indexes::Dense(range) => {
            range.start += 1;
            range.end += 1;
        }
    }
    res_p.provable_result = Some(provable_res_p);
    assert!(res_p.verify(expr, accessor, &()).is_err());

    // try to change data
    let mut res_p = res.clone();
    let mut provable_res_p = provable_res.clone();
    provable_res_p.data_mut()[0] += 1;
    res_p.provable_result = Some(provable_res_p);
    assert!(res_p.verify(expr, accessor, &()).is_err());
}
