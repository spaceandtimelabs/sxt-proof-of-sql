use crate::{
    base::proof::{Column, Commit, PipProve, PipVerify, Transcript},
    pip::positive::PositiveProof,
};

#[test]
fn test_positive_success() {
    let a: Column<i64> = vec![91_i64, 0_i64, -47_i64].into();
    let b: Column<bool> = vec![true, false, false].into();
    let c_a = a.commit();
    let mut transcript = Transcript::new(b"positivetest");
    let proof = PositiveProof::prove(&mut transcript, (a,), b.clone(), (c_a,));
    let mut transcript = Transcript::new(b"positivetest");
    assert!(proof.verify(&mut transcript, (c_a,)).is_ok());
    assert!(proof.get_output_commitments() == b.commit());
}

#[test]
fn test_positive_fail_zero() {
    let a: Column<i64> = vec![91_i64, 0_i64, -47_i64].into();
    let b: Column<bool> = vec![true, true, false].into();
    let c_a = a.commit();
    let mut transcript = Transcript::new(b"positivetest");
    let proof = PositiveProof::prove(&mut transcript, (a,), b, (c_a,));
    let mut transcript = Transcript::new(b"positivetest");
    assert!(proof.verify(&mut transcript, (c_a,)).is_err());
}

#[test]
fn test_positive_fail_transcript() {
    let a: Column<i64> = vec![91_i64, 0_i64, -47_i64].into();
    let b: Column<bool> = vec![true, false, false].into();
    let c_a = a.commit();
    let mut transcript = Transcript::new(b"positivetest");
    let proof = PositiveProof::prove(&mut transcript, (a,), b, (c_a,));
    let mut transcript = Transcript::new(b"oops");
    assert!(proof.verify(&mut transcript, (c_a,)).is_err());
}

#[test]
fn test_positive_fail_commitment() {
    let a: Column<i64> = vec![91_i64, 0_i64, -47_i64].into();
    let b: Column<bool> = vec![true, false, false].into();
    let c_a = a.commit();
    let mut transcript = Transcript::new(b"positivetest");
    let proof = PositiveProof::prove(&mut transcript, (a,), b.clone(), (c_a,));
    let mut transcript = Transcript::new(b"positivetest");
    assert!(proof.verify(&mut transcript, (b.commit(),)).is_err());
}
