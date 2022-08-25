use crate::base::proof::{Column, Commit, GeneralColumn, PipProve, PipVerify, Transcript};
use crate::pip::inequality::InequalityProof;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_inequality() {
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
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(2_u32),
        Scalar::from(3_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
    ]
    .into();

    let output: Column<bool> = vec![false, false, true, true, false, false, false].into();

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"inequalitytest");
    let inequalityproof =
        InequalityProof::prove(&mut transcript, (a, b), output.clone(), (c_a, c_b));

    let mut transcript = Transcript::new(b"inequalitytest");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(output.commit(), inequalityproof.get_output_commitments());

    let mut transcript = Transcript::new(b"inequalitytest oops");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_b)).is_err());

    //wrong input commitments
    let mut transcript = Transcript::new(b"inequalitytest");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_inequality_general() {
    let a = GeneralColumn::Int32Column(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::Int32Column(vec![1, -1, -2, 2, 3, 0].into());
    let output = GeneralColumn::BooleanColumn(vec![false, true, true, false, true, false].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"inequalitytest");
    let inequalityproof =
        InequalityProof::prove(&mut transcript, (a, b), output.clone(), (c_a, c_b));

    let mut transcript = Transcript::new(b"inequalitytest");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(output.commit(), inequalityproof.get_output_commitments());

    let mut transcript = Transcript::new(b"inequalitytest oops");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_b)).is_err());

    //wrong input commitments
    let mut transcript = Transcript::new(b"inequalitytest");
    assert!(inequalityproof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
#[should_panic]
fn test_inequality_general_mismatched_inputs() {
    let a = GeneralColumn::Int32Column(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::Int16Column(vec![1, -1, -2, 2, 3, 0].into());
    let output = GeneralColumn::BooleanColumn(vec![false, true, true, false, true, false].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"inequalitytest");
    let _should_panic = InequalityProof::prove(&mut transcript, (a, b), output, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_inequality_general_non_bool_output() {
    let a = GeneralColumn::Int32Column(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::Int32Column(vec![1, -1, -2, 2, 3, 0].into());
    let output = GeneralColumn::Int32Column(vec![0, 1, 1, 0, 1, 0].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"inequalitytest");
    let _should_panic = InequalityProof::prove(&mut transcript, (a, b), output, (c_a, c_b));
}
