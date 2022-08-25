use crate::{
    base::proof::{Column, Commit, GeneralColumn, PipProve, PipVerify, Transcript},
    pip::subtraction::SubtractionProof,
};
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_subtraction() {
    let a: Column<i32> = vec![1i32, 3i32, 3i32, 5i32, 0i32, 3i32].into();
    let b: Column<i32> = vec![4i32, 2i32, 4i32, 0i32, 2i32, 1i32].into();
    let diff: Column<i32> = vec![-3i32, 1i32, -1i32, 5i32, -2i32, 2i32].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_diff = diff.commit();

    let mut transcript = Transcript::new(b"subtractiontest");
    let proof = SubtractionProof::prove(&mut transcript, (a, b), diff, (c_a, c_b));

    let mut transcript = Transcript::new(b"subtractiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_diff);

    // wrong input commitments
    let mut transcript = Transcript::new(b"subtractiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_subtraction_wrong() {
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
    let diff: Column<Scalar> = vec![
        Scalar::from(0u32),
        Scalar::from(0u32),
        Scalar::from(0u32),
        Scalar::from(0u32),
        Scalar::from(0u32),
        Scalar::from(0u32),
    ]
    .into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_diff = diff.commit();

    let mut transcript = Transcript::new(b"subtractiontest");
    let proof = SubtractionProof::prove(&mut transcript, (a, b), diff, (c_a, c_b));

    assert_ne!(proof.get_output_commitments(), c_diff);
}

#[test]
fn test_subtraction_general() {
    let a = GeneralColumn::Int32Column(vec![1, 2, 3, 5, 0, 1].into());
    let b = GeneralColumn::Int32Column(vec![4, 3, 4, 0, 2, 3].into());
    let sum = GeneralColumn::Int32Column(vec![-3, -1, -1, 5, -2, -2].into());

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"subtractiontest");
    let proof = SubtractionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    let mut transcript = Transcript::new(b"subtractiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_sum);

    // wrong input commitments
    let mut transcript = Transcript::new(b"subtractiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
#[should_panic]
fn test_subtraction_general_mismatched_inputs() {
    let a = GeneralColumn::Int32Column(vec![1, 2, 3, 5, 0, 1].into());
    let b = GeneralColumn::Int16Column(vec![4, 3, 4, 0, 2, 3].into());
    let sum = GeneralColumn::Int32Column(vec![-3, -1, -1, 5, -2, -2].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"subtractiontest");
    let _should_panic = SubtractionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_subtraction_general_non_numeric() {
    let a = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let sum = GeneralColumn::BooleanColumn(vec![false, true, true, false, false, true].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"subtractiontest");
    let _should_panic = SubtractionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));
}
