use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::errors::ProofError;

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct MultiplicationProof {}

impl MultiplicationProof {
    /// Create a multiplication proof.
    ///
    /// See protocols/multiplication.pdf
    #[allow(unused_variables)]
    pub fn create(
        transcript: &mut Transcript,
        a_vec: &[Scalar],
        b_vec: &[Scalar],
    ) -> MultiplicationProof {
        let n = a_vec.len();

        assert_eq!(a_vec.len(), n);
        assert_eq!(b_vec.len(), n);

        MultiplicationProof {}
    }

    /// Verifies that a multiplication proof is correct given the associated commitments.
    #[allow(unused_variables)]
    pub fn verify(
        &self,
        transcript: &mut Transcript,
        commit_a: &RistrettoPoint,
        commit_b: &RistrettoPoint,
    ) -> Result<(), ProofError> {
        Ok(())
    }
}
