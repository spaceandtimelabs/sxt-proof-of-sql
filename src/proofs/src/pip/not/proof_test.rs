use crate::{
    base::proof::{Column, Commit, PipProve, PipVerify, Transcript},
    pip::not::NotProof,
};

#[test]
fn test_not_success() {
    let a: Column<bool> = vec![true, false, true, false, true, true].into();
    let not_a: Column<bool> = vec![false, true, false, true, false, false].into();
    let wrong_a: Column<bool> = vec![false, false, false, true, false, false].into();

    let c_a = a.commit();
    let c_not_a = not_a.commit();
    let c_wrong_a = wrong_a.commit();

    let mut p_transcript = Transcript::new(b"nottest");
    let proof = NotProof::prove(&mut p_transcript, (a,), not_a, (c_a,));

    let mut v_transcript = Transcript::new(b"nottest");
    assert!(proof.verify(&mut v_transcript, (c_a,)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_not_a);

    let mut v_transcript = Transcript::new(b"nottest");
    assert!(proof.verify(&mut v_transcript, (c_wrong_a,)).is_err());
}

#[test]
fn test_not_failure() {
    let a: Column<bool> = vec![true, false, true, false, true, true].into();
    let wrong_not_a: Column<bool> = vec![false, false, false, true, false, false].into();

    let c_a = a.commit();
    let c_wrong_not_a = wrong_not_a.commit();

    let mut p_transcript = Transcript::new(b"nottest");
    let proof = NotProof::prove(&mut p_transcript, (a,), wrong_not_a, (c_a,));

    assert_ne!(proof.get_output_commitments(), c_wrong_not_a);
}
