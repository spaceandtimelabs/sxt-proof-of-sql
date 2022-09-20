use crate::base::proof::{Column, Commit, GeneralColumn, PipProve, PipVerify, Transcript};
use crate::base::test_helpers::*;
use crate::pip::equality::EqualityProof;
use curve25519_dalek::scalar::Scalar;
use proptest::prelude::*;

proptest! {
    #[test]
    fn equality_on_scalars(
        p in arbitrary_column_array(1..=10, arbitrary_scalar())
    ) {
        let [a, b, c] = p; // Get three arbitrary columns of up to 10 elements each
        let len = a.len(); // Because a is consumed later
        let ab = a.iter().zip(b.iter()).map(|(x, y)| x == y).collect::<Column<bool>>();
        let not_ab = a.iter().zip(b.iter()).map(|(x, y)| x != y).collect::<Column<bool>>();

        let comm_a = a.commit();
        let comm_b = b.commit();
        let comm_c = c.commit();

        let proof = EqualityProof::prove(
            &mut Transcript::new(b"equalitytest"),
            (a.clone(), b.clone()),
            ab.clone(),
            (comm_a, comm_b),
        );

        proof.verify(
            &mut Transcript::new(b"equalitytest"),
            (comm_a, comm_b),
        ).expect("Valid proof should verify");

        if len > 1 {
            // TODO: Why does len have to be > 1 for this to work?
            proof.verify(
                &mut Transcript::new(b"invalid"),
                (comm_a, comm_b),
            ).expect_err("Should not be able to change Transcript domain separator");
        }

        proof.verify(
            &mut Transcript::new(b"equalitytest"),
            (comm_a, comm_c),
        ).expect_err("Should not be able to change input commitments");

        {
            let proof = EqualityProof::prove(
                &mut Transcript::new(b"equalitytest"),
                (a.clone(), b.clone()),
                not_ab.clone(),
                (comm_a, comm_b),
            );
            proof.verify(
                &mut Transcript::new(b"equalitytest"),
                (comm_a, comm_b),
            ).expect_err("Should not be able to change output");
        }
    }
}

#[test]
fn test_equality() {
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

    let output: Column<bool> = vec![true, true, false, false, true, true, true].into();

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"equalitytest");
    let equalityproof = EqualityProof::prove(&mut transcript, (a, b), output.clone(), (c_a, c_b));

    //the proof confirms as correct
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_ok());

    //the output commitment is correct as well
    assert_eq!(output.commit(), equalityproof.get_output_commitments());

    //wrong transcript
    let mut transcript = Transcript::new(b"equalitytest oops");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_err());

    //wrong input commitments
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_equality_non_scalar() {
    // Test equality for u32s
    let a: Column<u32> = vec![1, 2, 2, 3, 3, 4, 4].into();
    let b: Column<u32> = vec![1, 1, 2, 2, 3, 3, 4].into();

    let c_a = a.commit();
    let c_b = b.commit();

    let eq_a_b: Column<bool> = vec![true, false, true, false, true, false, true].into();
    let c_eq_a_b = eq_a_b.commit();

    let mut transcript = Transcript::new(b"equalitytest");

    let eq_a_b_proof = EqualityProof::prove(&mut transcript, (a, b), eq_a_b.clone(), (c_a, c_b));

    let mut transcript = Transcript::new(b"equalitytest");

    assert!(eq_a_b_proof.verify(&mut transcript, (c_a, c_b)).is_ok());
    assert_eq!(c_eq_a_b, eq_a_b_proof.get_output_commitments());

    // Test equality for bools, using the previous output as one of the inputs
    let c: Column<bool> = vec![true, true, false, false, true, true, false].into();
    let c_c = c.commit();

    let eq_eq_a_b_c: Column<bool> = vec![true, false, false, true, true, false, false].into();
    let c_eq_eq_a_b_c = eq_eq_a_b_c.commit();

    let mut transcript = Transcript::new(b"equalitytest");
    let eq_eq_a_b_c_proof =
        EqualityProof::prove(&mut transcript, (eq_a_b, c), eq_eq_a_b_c, (c_eq_a_b, c_c));

    let mut transcript = Transcript::new(b"equalitytest");
    assert!(eq_eq_a_b_c_proof
        .verify(&mut transcript, (c_eq_a_b, c_c))
        .is_ok());
    assert_eq!(c_eq_eq_a_b_c, eq_eq_a_b_c_proof.get_output_commitments());
}

#[test]
fn test_equality_general() {
    let a = GeneralColumn::SafeIntColumn(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::SafeIntColumn(vec![1, -1, -2, 2, 3, 0].into());
    let output = GeneralColumn::BooleanColumn(vec![true, false, false, true, false, true].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"equalitytest");
    let equalityproof = EqualityProof::prove(&mut transcript, (a, b), output.clone(), (c_a, c_b));

    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(output.commit(), equalityproof.get_output_commitments());

    let mut transcript = Transcript::new(b"equalitytest oops");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_err());

    // wrong input commitments
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_a)).is_err());

    // equalityproof commitments and input commitments mismatch in length
    let mut c_a = c_a;
    c_a.length += 1;
    let mut c_b = c_b;
    c_b.length += 1;
    let mut transcript = Transcript::new(b"equalitytest");
    assert!(equalityproof.verify(&mut transcript, (c_a, c_b)).is_err());
}

#[test]
#[should_panic]
fn test_equality_general_mismatched_inputs() {
    let a = GeneralColumn::SafeIntColumn(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let output = GeneralColumn::BooleanColumn(vec![true, false, false, false, false, true].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"equalitytest");
    let _should_panic = EqualityProof::prove(&mut transcript, (a, b), output, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_equality_general_non_bool_output() {
    let a = GeneralColumn::SafeIntColumn(vec![1, 1, 2, 2, 0, 0].into());
    let b = GeneralColumn::SafeIntColumn(vec![1, -1, -2, 2, 3, 0].into());
    let output = GeneralColumn::SafeIntColumn(vec![1, 0, 0, 1, 0, 1].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"equalitytest");
    let _should_panic = EqualityProof::prove(&mut transcript, (a, b), output, (c_a, c_b));
}
