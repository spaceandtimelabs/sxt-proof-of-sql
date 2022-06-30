use crate::base::proof::{Commitment, PIPProof, Transcript};
use crate::pip::equality::EqualityProof;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_equality() {
    let a = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
    ];
    let b = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(2_u32),
        Scalar::from(3_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
    ];

    let output = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(0_u32),
        Scalar::from(0_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
    ];

    let c_a = Commitment::from(&a[..]);
    let c_b = Commitment::from(&b[..]);

    let mut transcript = Transcript::new(b"equalitytest");
    let equalityproof = EqualityProof::create(&mut transcript, &[&a, &b], &[&output], &[c_a, c_b]);

    //the proof confirms as correct
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, &[c_a, c_b]).is_ok());

    //the output commitment is correct as well
    assert_eq!(
        Commitment::from(&output[..]),
        equalityproof.get_output_commitments()[0]
    );

    //wrong transcript
    let mut transcript = Transcript::new(b"equalitytest oops");
    assert!(equalityproof.verify(&mut transcript, &[c_a, c_b]).is_err());

    //wrong input commitments
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, &[c_a, c_a]).is_err());
}
