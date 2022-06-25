use std::slice;
use curve25519_dalek::{scalar::Scalar, ristretto::CompressedRistretto, traits::Identity};
use pedersen::commitments::compute_commitments;
use crate::base::proof::{Commitment, ProofError};

pub struct HadamardProof {
    pub a : Vec<Scalar>,
    pub b : Vec<Scalar>,
    pub c : Vec<Scalar>
}

impl HadamardProof {
    pub fn create_hadamard_proof(a : &[Scalar], b : &[Scalar], c : &[Scalar]) -> Self {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), c.len());
        HadamardProof { a:a.to_vec(), b:b.to_vec(), c:c.to_vec() }
    }
    pub fn verify(&self, c_a: Commitment, c_b: Commitment, c_c: Commitment) -> Result<(), ProofError> {
        assert_eq!(self.a.len(), self.b.len());
        assert_eq!(self.a.len(), self.c.len());
        let mut valid = true;
        for i in 0..self.a.len() {
            if self.a[i] * self.b[i] != self.c[i] {
                valid = false;
            }
        }
/*
        println!("========================================================");
        println!("{}", valid);
        println!("{}", C_a == pedersen::pedersen_commitment(&self.a));
        println!("{}", C_b == pedersen::pedersen_commitment(&self.b));
        println!("{}", C_c == pedersen::pedersen_commitment(&self.c));
        println!("========================================================");
*/

        let mut commitment_a = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut commitment_a), &[&self.a[..]]);
        let mut commitment_b = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut commitment_b), &[&self.b[..]]);
        let mut commitment_c = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut commitment_c), &[&self.c[..]]);


        valid = valid &&
        c_a.commitment == commitment_a &&
        c_b.commitment == commitment_b &&
        c_c.commitment == commitment_c &&
        c_a.length == self.a.len() &&
        c_b.length == self.b.len() &&
        c_c.length == self.c.len();
        if valid {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }
}