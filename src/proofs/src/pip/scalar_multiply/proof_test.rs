use crate::{
    base::proof::{Column, Commit, PipProve, PipVerify, Transcript},
    pip::scalar_multiply::ScalarMultiplyProof,
};

#[test]
fn test_and_success() {
    let a: Column<bool> = vec![true, false, true, true, true, false].into();
    let b: Column<bool> = vec![false, true, true, true, false, false].into();
    let c: Column<bool> = vec![false, false, true, true, false, false].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_c = c.commit();

    let mut p_transcript = Transcript::new(b"andtest");
    let proof = ScalarMultiplyProof::prove(&mut p_transcript, (a, b), c, (c_a, c_b));

    let mut v_transcript = Transcript::new(b"andtest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_b)).is_ok());

    assert_eq!(proof.get_output_commitments(), c_c);
}
