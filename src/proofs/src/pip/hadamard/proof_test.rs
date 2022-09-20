use crate::pip::hadamard::proof::*;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use proptest::prelude::*;

use crate::base::proof::{Column, Commit, Commitment, PipProve, PipVerify, Transcript};
use crate::base::test_helpers::*;

proptest! {
    #[test]
    fn hadamard_properties(
        p in arbitrary_column_array(1..=10, arbitrary_scalar())
    ) {
        let [a, b, c] = p; // Get three arbitrary columns of up to 10 elements each
        let len = a.len(); // Because a is consumed later
        let ab = &a * &b;

        let comm_a = a.commit();
        let comm_b = b.commit();
        let comm_c = c.commit();

        let proof = HadamardProof::prove(
            &mut Transcript::new(b"hadamardtest"),
            (a.clone(), b.clone()),
            ab.clone(),
            (comm_a, comm_b),
        );

        proof.verify(
            &mut Transcript::new(b"hadamardtest"),
            (comm_a, comm_b),
        ).expect("Valid proof should verify");

        if len > 1 {
            // TODO: Why does len have to be > 1 for this to work?
            proof.verify(
                &mut Transcript::new(b"invalid"),
                (comm_a, comm_b),
            ).expect_err("Should not be able to change Transcript domain separator");
        }
        {
            let mut proof = proof.clone();
            proof.f_a = Scalar::one();
            proof.verify(
                &mut Transcript::new(b"hadamardtest"),
                (comm_a, comm_c),
            ).expect_err("Should not be able to change f_a");
        }
        {
            let mut proof = proof.clone();
            proof.f_b = Scalar::one();
            proof.verify(
                &mut Transcript::new(b"hadamardtest"),
                (comm_a, comm_c),
            ).expect_err("Should not be able to change f_b");
        }
        {
            let proof = HadamardProof::prove(
                &mut Transcript::new(b"hadamardtest"),
                (a.clone(), b.clone()),
                a.clone(),
                (comm_a, comm_b),
            );
            proof.verify(
                &mut Transcript::new(b"hadamardtest"),
                (comm_a, comm_b),
            ).expect_err("Should not be able to change output");
        }
    }
}

/// This test should already be covered by the property test
#[test]
fn test_zero_proof() {
    let a_vec = Column::from(vec![Scalar::zero()]);

    let commitment = Commitment::from_compressed(CompressedRistretto::identity(), a_vec.len());

    let mut transcript = Transcript::new(b"hadamardtest");
    let proof = HadamardProof::prove(
        &mut transcript,
        (a_vec.clone(), a_vec.clone()),
        a_vec.into(),
        (commitment, commitment),
    );

    // verify proof
    let mut transcript = Transcript::new(b"hadamardtest");
    assert!(proof
        .verify(&mut transcript, (commitment, commitment))
        .is_ok());
}
