use crate::{
    base::proof::{Column, Commit, PipProve, PipVerify, Transcript},
    pip::addition::AdditionProof,
};
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_addition() {
    let a: Column<u32> = vec![1u32, 2u32, 3u32, 5u32, 0u32, 1u32].into();
    let b: Column<u32> = vec![4u32, 3u32, 4u32, 0u32, 2u32, 3u32].into();
    let sum: Column<u32> = vec![5u32, 5u32, 7u32, 5u32, 2u32, 4u32].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_sum);

    // wrong input commitments
    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_addition_wrong() {
    let a: Column<Scalar> = vec![
        Scalar::from(1u32),
        Scalar::from(2u32),
        Scalar::from(3u32),
        Scalar::from(5u32),
        Scalar::from(0u32),
        Scalar::from(1u32),
    ]
    .into();
    let b: Column<Scalar> = vec![
        Scalar::from(1u32),
        Scalar::from(1u32),
        Scalar::from(1u32),
        Scalar::from(1u32),
        Scalar::from(1u32),
        Scalar::from(1u32),
    ]
    .into();
    let sum: Column<Scalar> = vec![
        Scalar::from(2u32),
        Scalar::from(3u32),
        Scalar::from(4u32),
        Scalar::from(5u32),
        Scalar::from(1u32),
        Scalar::from(2u32),
    ]
    .into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    assert_ne!(proof.get_output_commitments(), c_sum);
}
