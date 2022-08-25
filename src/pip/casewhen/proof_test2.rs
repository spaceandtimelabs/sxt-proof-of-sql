use crate::base::proof::{Column, Commit, GeneralColumn, PipProve, PipVerify, Transcript};
use crate::pip::casewhen::CaseWhenProof;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_casewhen() {
    let a_vec: Column<i32> = vec![31, 24, 51].into();
    let b_vec: Column<i32> = vec![14, 23, 71].into();
    let c_vec: Column<i32> = vec![31, 23, 71].into();
    let a: GeneralColumn = GeneralColumn::Int32Column(a_vec);
    let b: GeneralColumn = GeneralColumn::Int32Column(b_vec);
    let c: GeneralColumn = GeneralColumn::Int32Column(c_vec);
    let p: Column<bool> = vec![true, true, false].into();
    let p_scalar: Column<Scalar> = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(0_u32),
    ]
    .into();
    let c_a = a.commit();
    let c_b = b.commit();
    let c_p = p_scalar.commit();

    let mut transcript = Transcript::new(b"casewhentest");
    let proof = CaseWhenProof::prove(&mut transcript, (a, b, p), c, (c_a, c_b, c_p));

    //wrong input commitments
    let mut transcript = Transcript::new(b"casewhentest");
    assert!(proof.verify(&mut transcript, (c_a, c_a, c_p)).is_err());
}
