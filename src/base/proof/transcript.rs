use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

pub trait TranscriptProtocol {
    /// Append a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);

    /// Append a `point` with the given `label`.
    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto);

    /// Compute a `label`ed challenge variable.
    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: &'static [u8]);
}

impl TranscriptProtocol for Transcript {
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes());
    }

    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto) {
        self.append_message(label, point.as_bytes());
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
