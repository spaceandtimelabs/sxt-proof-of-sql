use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;

use crate::base::proof::{Commitment, ProofError};
use crate::base::scalar::as_byte_slice;

// Note: for background on label and domain usage, see
//      https://merlin.cool/use/passing.html#sequential-composition
pub struct Transcript(merlin::Transcript);

impl Transcript {
    /// Initialize a new transcript with the supplied `label`, which
    /// is used as a domain separator.
    ///
    /// # Note
    ///
    /// This function should be called by a proof library's API
    /// consumer (i.e., the application using the proof library), and
    /// **not by the proof implementation**.  See the [Passing
    /// Transcripts](https://merlin.cool/use/passing.html) section of
    /// the Merlin website for more details on why.
    pub fn new(label: &'static [u8]) -> Transcript {
        Transcript(merlin::Transcript::new(label))
    }

    /// Append a domain separator for a equality proof of length n
    pub fn equality_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"equalityproof v1");
        self.0.append_u64(b"n", n);
    }

    /// Append a domain separator for a subtraction proof of length n
    pub fn subtraction_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"subtractionproof v1");
        self.0.append_u64(b"n", n);
    }

    /// Append a domain separator for a hadamard proof with n variables
    pub fn hadamard_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"hadamardproof v1");
        self.0.append_u64(b"n", n);
    }

    /// Append a domain separator for a sumcheck proof with m multiplicands and n variables
    pub fn sumcheck_domain_sep(&mut self, m: u64, n: u64) {
        self.0.append_message(b"dom-sep", b"sumcheckproof v1");
        self.0.append_u64(b"m", m);
        self.0.append_u64(b"n", n);
    }

    /// Append a domain separator for an addition proof of length n
    pub fn addition_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"additionproof v1");
        self.0.append_u64(b"n", n);
    }

    /// Append a domain separator for a not proof of length n
    pub fn not_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"notproof v1");
        self.0.append_u64(b"n", n);
    }

    /// Append a `scalar` with the given `label`.
    pub fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.0.append_message(label, scalar.as_bytes());
    }

    /// Append a `scalars` with the given `label`.
    pub fn append_scalars(&mut self, label: &'static [u8], scalars: &[Scalar]) {
        self.0.append_message(label, as_byte_slice(scalars));
    }

    /// Append a `point` with the given `label`.
    pub fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto) {
        self.0.append_message(label, point.as_bytes());
    }

    /// Append a `commitment` with the given `label`.
    pub fn append_commitment(&mut self, label: &'static [u8], commitment: &Commitment) {
        self.0
            .append_message(label, commitment.commitment.as_bytes());
    }

    /// Append a slice of `commitments` with the given `label`.
    pub fn append_commitments(&mut self, label: &'static [u8], commitments: &[Commitment]) {
        for c in commitments {
            self.append_commitment(label, c);
        }
    }

    /// Check that a point is not the identity, then append it to the
    /// transcript.  Otherwise, return an error.
    pub fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) -> Result<(), ProofError> {
        use curve25519_dalek::traits::IsIdentity;

        if point.is_identity() {
            Err(ProofError::VerificationError)
        } else {
            self.0.append_message(label, point.as_bytes());
            Ok(())
        }
    }

    /// Compute a `label`ed challenge variable.
    pub fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar {
        let mut buf = [0u8; 64];
        self.0.challenge_bytes(label, &mut buf);

        Scalar::from_bytes_mod_order_wide(&buf)
    }

    /// Compute a `label`ed challenge variable.
    pub fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: &'static [u8]) {
        let n = scalars.len();
        assert!(n > 0);

        let mut buf = vec![0u8; n * 64];
        self.0.challenge_bytes(label, &mut buf);
        for (i, scalar) in scalars.iter_mut().enumerate().take(n) {
            let s = i * 64;
            let t = s + 64;

            let bytes: [u8; 64] = buf[s..t].try_into().unwrap();
            *scalar = Scalar::from_bytes_mod_order_wide(&bytes);
        }
    }

    pub fn innerproduct_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"ipp v1");
        self.0.append_u64(b"n", n);
    }

    pub fn negative_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"negative v1");
    }

    pub fn column_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"column v1");
    }

    pub fn literal_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"literal v1");
    }

    pub fn projection_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"projection v1");
    }

    pub fn scalar_multiply_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"scalarmultiplyproof v1");
    }

    pub fn or_domain_sep(&mut self, n: u64) {
        self.0.append_message(b"dom-sep", b"orproof v1");
        self.0.append_u64(b"n", n);
    }

    pub fn trivial_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"trivial v1");
    }

    pub fn reader_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"reader v1");
    }

    pub fn count_domain_sep(&mut self) {
        self.0.append_message(b"dom-sep", b"count v1");
    }
}
