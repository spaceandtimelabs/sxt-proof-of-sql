use crate::base::proof::{Column, Commit, PipProve, PipVerify, Transcript};
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
