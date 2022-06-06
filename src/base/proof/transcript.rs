use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::base::proof::ProofError;

// Note: for background on label and domain usage, see
//      https://merlin.cool/use/passing.html#sequential-composition
pub trait TranscriptProtocol {
    /// Append a domain separator for a multiplication proof with n variables
    fn multiplication_domain_sep(&mut self, n: u64);

    /// Append a domain separator for a multiplication proof with m multiplcands and n variables
    fn sumcheck_domain_sep(&mut self, m: u64, n: u64);

    /// Append a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);

    /// Append a `point` with the given `label`.
    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto);

    /// Compute a `label`ed challenge variable.
    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: &'static [u8]);

    /// Check that a point is not the identity, then append it to the
    /// transcript.  Otherwise, return an error.
    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) -> Result<(), ProofError>;
}

impl TranscriptProtocol for Transcript {
    fn multiplication_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"multiplicationproof v1");
        self.append_u64(b"n", n);
    }

    fn sumcheck_domain_sep(&mut self, m: u64, n: u64) {
        self.append_message(b"dom-sep", b"sumcheckproof v1");
        self.append_u64(b"m", m);
        self.append_u64(b"n", n);
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes());
    }

    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto) {
        self.append_message(label, point.as_bytes());
    }

    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) -> Result<(), ProofError> {
        use curve25519_dalek::traits::IsIdentity;

        if point.is_identity() {
            Err(ProofError::VerificationError)
        } else {
            Ok(self.append_message(label, point.as_bytes()))
        }
    }

    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: &'static [u8]) {
        let n = scalars.len();
        assert!(n > 0);

        let mut buf = vec![0u8; n * 64];
        self.challenge_bytes(label, &mut buf);
        for i in 0..n {
            let s = i * 64;
            let t = s + 64;
            let bytes: [u8; 64];
            bytes = buf[s..t].try_into().unwrap();
            scalars[i] = Scalar::from_bytes_mod_order_wide(&bytes);
        }
    }
}
