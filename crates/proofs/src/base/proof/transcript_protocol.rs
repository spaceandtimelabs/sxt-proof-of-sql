use crate::base::scalar::as_byte_slice;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

/// Merlin Transcripts, for non-interactive proofs.
///
/// Think of this as a transaction log.
/// When adding data to it, the opaque state will change in a pseudorandom way.
/// The provers and verifiers should be able to add the same data in the same order, and achieve the same state.
///
/// In most cases, you would use this in PipVerify and PipProve, where they would look like this:
/// ```ignore
/// use proofs::base::proof::{Commitment, MessageLabel, transcript::Transcript};
/// use curve25519_dalek::scalar::Scalar;
/// let mut transcript = Transcript::new(b"my-protocol-name");
///
/// // Begin your proof with message specific to your operation using the MessageLabel enum.
/// // Still, keep them simple and small.
/// let param = 8u64; // This would not normally be constant
/// // let param = &(8u64, 9u64); // any serializable type can accompany the label, but keep it as simple as possible
///
/// // Actually implement the proof .. (not shown)
///
/// let c_c = Commitment::from(&[] as &[Scalar]); // these would not normally be empty
/// let c_e = Commitment::from(&[] as &[Scalar]);
///
/// // Now include the operation, its parameters, and the commitments in the transcript
/// transcript.append_auto(MessageLabel::MyProofLabel1, &(
///     param,
///     c_c,
///     c_e,
/// ));
///
/// // It's completely valid to include additional addenda to the transcript, but for many operations, this is all you need.
/// ```
pub trait TranscriptProtocol {
    /// Append a message to the transcript, automatically serializing it.
    /// Usually you would use this to start an operation, as well as include a few commitments.
    /// When including commitments this way, besure to use Commitment::as_compressed(),
    /// so that only the RistrettoPoint is included, and not the length.
    ///
    /// The message is encoded with Postcard v1, chosen for its simplicity and stability.
    fn append_auto(&mut self, label: MessageLabel, message: &impl serde::Serialize);

    /// Append some scalars to the transcript under a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto.
    /// But Scalars are not Serialize, so you must use this method instead, creating a separate message.
    fn append_scalars(&mut self, label: MessageLabel, scalars: &[Scalar]);

    /// Append a Compressed RistrettoPoint with a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto instead,
    /// because using this method creates a need for more labels
    fn append_point(&mut self, label: MessageLabel, point: &CompressedRistretto);

    /// Append Compressed RistrettoPoint's with a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto instead,
    /// because using this method creates a need for more labels
    fn append_points(&mut self, label: MessageLabel, points: &[CompressedRistretto]);

    /// Compute a challenge variable (which requires a label).
    fn challenge_scalar(&mut self, label: MessageLabel) -> Scalar;

    /// Compute multiple challenge variables (which requires a label).
    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: MessageLabel);
}

impl TranscriptProtocol for Transcript {
    fn append_auto(&mut self, label: MessageLabel, message: &impl serde::Serialize) {
        self.append_message(label.as_bytes(), &postcard::to_allocvec(message).unwrap());
    }

    fn append_scalars(&mut self, label: MessageLabel, scalars: &[Scalar]) {
        self.append_message(label.as_bytes(), as_byte_slice(scalars));
    }

    fn append_point(&mut self, label: MessageLabel, point: &CompressedRistretto) {
        self.append_message(label.as_bytes(), point.as_bytes());
    }

    fn append_points(&mut self, label: MessageLabel, points: &[CompressedRistretto]) {
        self.append_message(label.as_bytes(), points_as_byte_slice(points));
    }

    fn challenge_scalar(&mut self, label: MessageLabel) -> Scalar {
        let mut buf = [0u8; 64];
        self.challenge_bytes(label.as_bytes(), &mut buf);

        Scalar::from_bytes_mod_order_wide(&buf)
    }

    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: MessageLabel) {
        let n = scalars.len();
        assert!(n > 0);

        let mut buf = vec![0u8; n * 64];
        self.challenge_bytes(label.as_bytes(), &mut buf);
        for (i, scalar) in scalars.iter_mut().enumerate().take(n) {
            let s = i * 64;
            let t = s + 64;

            let bytes: [u8; 64] = buf[s..t].try_into().unwrap();
            *scalar = Scalar::from_bytes_mod_order_wide(&bytes);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLabel {
    InnerProduct,
    InnerProductChallenge,
    InnerProductLeft,
    InnerProductRight,
    Sumcheck,
    SumcheckChallenge,
    SumcheckRoundEvaluation,
}
impl MessageLabel {
    /// Convert the label to a byte slice, which satisfies the requirements of a merlin label:
    /// "the labels should be distinct and none should be a prefix of any other."
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            MessageLabel::InnerProduct => b"ipp v1",
            MessageLabel::InnerProductChallenge => b"ippchallenge v1",
            MessageLabel::InnerProductLeft => b"ippleft v1",
            MessageLabel::InnerProductRight => b"ippright v1",
            MessageLabel::Sumcheck => b"sumcheckproof v1",
            MessageLabel::SumcheckChallenge => b"sumcheckchallenge v1",
            MessageLabel::SumcheckRoundEvaluation => b"sumcheckroundevaluationscalars v1",
        }
    }
}

fn points_as_byte_slice(slice: &[CompressedRistretto]) -> &[u8] {
    let slice = slice;
    let len = slice.len() * core::mem::size_of::<CompressedRistretto>();
    unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}
