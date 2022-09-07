use crate::base::{
    proof::{Commit, GeneralColumn, PipProve, PipVerify, Transcript},
    scalar::SafeIntColumn,
};
use crate::pip::casewhen::CaseWhenProof;
use curve25519_dalek::scalar::Scalar;

//This test is for a valid case.
#[test]
fn test_casewhen() {
    let a_vec: SafeIntColumn = vec![31, 24, 51].into();
    let b_vec: SafeIntColumn = vec![14, 23, 71].into();
    let c_vec: SafeIntColumn = vec![31, 23, 71].into();
    let a: GeneralColumn = GeneralColumn::SafeIntColumn(a_vec);
    let b: GeneralColumn = GeneralColumn::SafeIntColumn(b_vec);
    let c: GeneralColumn = GeneralColumn::SafeIntColumn(c_vec);
    let p = GeneralColumn::BooleanColumn(vec![true, false, false].into());
    let p_numeric =
        SafeIntColumn::try_new(vec![Scalar::one(), Scalar::zero(), Scalar::zero()], 0).unwrap();
    let c_a = a.commit();
    let c_b = b.commit();
    let c_p = p_numeric.commit();
    let c_c = c.commit();

    let mut transcript = Transcript::new(b"casewhentest");
    let proof = CaseWhenProof::prove(&mut transcript, (a, b, p), c, (c_a, c_b, c_p));

    //the proof confirms as correct
    let mut transcript = Transcript::new(b"casewhentest");
    assert!(proof.verify(&mut transcript, (c_a, c_b, c_p)).is_ok());

    //the output commitment is correct as well
    assert_eq!(c_c, proof.get_output_commitments());

    //wrong transcript
    let mut transcript = Transcript::new(b"casewhen oops");
    assert!(proof.verify(&mut transcript, (c_a, c_b, c_p)).is_err());

    //wrong input commitments
    let mut transcript = Transcript::new(b"casewhentest");
    assert!(proof.verify(&mut transcript, (c_a, c_a, c_p)).is_err());
}
