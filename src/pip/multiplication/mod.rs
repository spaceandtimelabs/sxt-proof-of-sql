use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use sha3::Sha3_512;

use crate::base::proof::ProofError;
use crate::base::math::log2_up;

mod sumcheck_polynomial;
#[cfg(test)]
mod sumcheck_polynomial_test;

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct MultiplicationProof {
    pub commit_ab: RistrettoPoint,
}

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
        assert!(n > 0);
        assert_eq!(a_vec.len(), n);
        assert_eq!(b_vec.len(), n);

        let c_ab = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"a"); // pretend like this is the commitment of ab

        MultiplicationProof {
            commit_ab: c_ab,
        }
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
