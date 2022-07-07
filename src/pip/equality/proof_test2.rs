use crate::base::proof::{Column, Commit, PipProve, PipVerify, Transcript};
use crate::pip::equality::EqualityProof;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_equality() {
    let a: Column<Scalar> = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
    ]
    .into();

    let b: Column<Scalar> = vec![
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
    ]
    .into();

    let output: Column<bool> = vec![false, false, false, false, true, false, false].into();

    let c_a = a.commit();
    let c_b = b.commit();

    //wrong output
    let mut transcript = Transcript::new(b"equalitytest");
    let equalityproof =
        EqualityProof::prove(&mut transcript, (a.into(), b.into()), output, (c_a, c_b));

    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_err());
}

#[test]
fn test_wrong_equality_non_scalar() {
    let a: Column<u32> = vec![1, 1, 2, 2, 1, 1].into();
    let b: Column<u32> = vec![1, 2, 1, 2, 1, 1].into();

    let c_a = a.commit();
    let c_b = b.commit();

    let wrong_eq_a_b: Column<bool> = vec![true, false, false, true, true, false].into();

    let mut transcript = Transcript::new(b"equalitytest");
    let proof_wrong_eq_a_b =
        EqualityProof::prove(&mut transcript, (a, b), wrong_eq_a_b, (c_a, c_b));

    let mut transcript = Transcript::new(b"equalitytest");
    assert!(proof_wrong_eq_a_b
        .verify(&mut transcript, (c_a, c_b))
        .is_err());
}
