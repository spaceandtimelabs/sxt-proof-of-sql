use crate::{
    base::proof::{Column, Commit, GeneralColumn, PipProve, PipVerify, Transcript},
    pip::or::OrProof,
};

#[test]
fn test_or_success() {
    let a: Column<bool> = vec![true, false, true, true, true, false].into();
    let b: Column<bool> = vec![false, true, true, true, false, false].into();
    let c: Column<bool> = vec![true, true, true, true, true, false].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_c = c.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let proof = OrProof::prove(&mut p_transcript, (a, b), c, (c_a, c_b));

    let mut v_transcript = Transcript::new(b"ortest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_b)).is_ok());
    assert_eq!(proof.get_output_commitments(), c_c);

    let mut v_transcript = Transcript::new(b"ortest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_or_failure1() {
    let a: Column<bool> = vec![true, false, true, true, true, false].into();
    let b: Column<bool> = vec![false, true, true, true, false, false].into();
    let c: Column<bool> = vec![true, true, true, true, true, false].into();
    let d: Column<bool> = vec![false, false, true, true, true, true].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_d = d.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let proof = OrProof::prove(&mut p_transcript, (a, b.clone()), c.clone(), (c_a, c_b));

    let mut v_transcript = Transcript::new(b"ortest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_d)).is_err());
}

#[test]
fn test_or_failure2() {
    let a: Column<bool> = vec![true, false, true, true, true, false].into();
    let b: Column<bool> = vec![false, true, true, true, false, false].into();
    let c: Column<bool> = vec![false, false, true, true, true, false].into();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_c = c.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let proof = OrProof::prove(&mut p_transcript, (a, b.clone()), c.clone(), (c_a, c_b));

    assert_ne!(proof.get_output_commitments(), c_c);
}

#[test]
fn test_or_general() {
    let a = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let c = GeneralColumn::BooleanColumn(vec![true, true, true, false, true, true].into());

    let c_a = a.commit();
    let c_b = b.commit();
    let c_c = c.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let proof = OrProof::prove(&mut p_transcript, (a, b), c, (c_a, c_b));

    let mut v_transcript = Transcript::new(b"ortest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_b)).is_ok());
    assert_eq!(proof.get_output_commitments(), c_c);

    let mut v_transcript = Transcript::new(b"ortest");
    assert!(proof.verify(&mut v_transcript, (c_a, c_a)).is_err());
}

#[test]
#[should_panic]
fn test_or_general_non_bool_input() {
    let a = GeneralColumn::SafeIntColumn(vec![1, 1, 0, 0, 1, 1].into());
    let b = GeneralColumn::SafeIntColumn(vec![1, 0, 1, 0, 1, 0].into());
    let c = GeneralColumn::BooleanColumn(vec![true, true, true, false, true, true].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let _should_panic = OrProof::prove(&mut p_transcript, (a, b), c, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_or_general_non_bool_output() {
    let a = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let c = GeneralColumn::SafeIntColumn(vec![1, 1, 1, 0, 1, 1].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut p_transcript = Transcript::new(b"ortest");
    let _should_panic = OrProof::prove(&mut p_transcript, (a, b), c, (c_a, c_b));
}
