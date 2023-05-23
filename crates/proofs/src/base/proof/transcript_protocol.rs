use crate::base::polynomial::ArkScalar;
use crate::base::polynomial::Scalar;
use crate::base::scalar::ToArkScalar;
use crate::base::slice_ops;
use ark_serialize::CanonicalSerialize;
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;

/// Merlin Transcripts, for non-interactive proofs.
///
/// Think of this as a transaction log.
/// When adding data to it, the opaque state will change in a pseudorandom way.
/// The provers and verifiers should be able to add the same data in the same order, and achieve the same state.
///
/// In most cases it looks like this:
/// ```ignore
/// use proofs::base::proof::{Commitment, MessageLabel, transcript::Transcript};
/// use crate::base::polynomial::Scalar;
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
    /// But ArkScalars are not Serialize, so you must use this method instead, creating a separate message.
    fn append_ark_scalars(&mut self, label: MessageLabel, scalars: &[ArkScalar]);

    /// Append some scalars to the transcript under a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto.
    /// But Scalars are not Serialize, so you must use this method instead, creating a separate message.
    fn append_scalars(&mut self, label: MessageLabel, scalars: &[Scalar]) {
        self.append_ark_scalars(
            label,
            &slice_ops::slice_cast_with(scalars, ToArkScalar::to_ark_scalar),
        );
    }

    /// Append a Compressed RistrettoPoint with a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto instead,
    /// because using this method creates a need for more labels
    fn append_point(&mut self, label: MessageLabel, point: &CompressedRistretto) {
        self.append_points(label, core::slice::from_ref(point));
    }

    /// Append Compressed RistrettoPoint's with a specific label.
    ///
    /// For most types, prefer to include it as part of the message with append_auto instead,
    /// because using this method creates a need for more labels
    fn append_points(&mut self, label: MessageLabel, points: &[CompressedRistretto]);

    /// Compute a challenge variable (which requires a label).
    fn challenge_scalar(&mut self, label: MessageLabel) -> Scalar {
        let mut buf = [Default::default(); 1];
        self.challenge_scalars(&mut buf, label);
        buf[0]
    }

    /// Compute multiple challenge variables (which requires a label).
    fn challenge_scalars(&mut self, scalars: &mut [Scalar], label: MessageLabel) {
        let mut buf = vec![Default::default(); scalars.len()];
        self.challenge_ark_scalars(&mut buf, label);
        for (scalar, ark_scalar) in scalars.iter_mut().zip(buf.iter()) {
            *scalar = ark_scalar.into_scalar();
        }
    }

    /// Compute multiple challenge variables (which requires a label).
    fn challenge_ark_scalars(&mut self, scalars: &mut [ArkScalar], label: MessageLabel);

    /// Compute a challenge variable (which requires a label).
    fn challenge_ark_scalar(&mut self, label: MessageLabel) -> ArkScalar {
        let mut buf = [Default::default(); 1];
        self.challenge_ark_scalars(&mut buf, label);
        buf[0]
    }
}

impl TranscriptProtocol for Transcript {
    fn append_auto(&mut self, label: MessageLabel, message: &impl serde::Serialize) {
        self.append_message(label.as_bytes(), &postcard::to_allocvec(message).unwrap());
    }

    fn append_ark_scalars(&mut self, label: MessageLabel, scalars: &[ArkScalar]) {
        let mut buf = vec![Default::default(); scalars.compressed_size()];
        scalars.serialize_compressed(&mut buf).unwrap();
        self.append_message(label.as_bytes(), &buf);
    }

    fn append_points(&mut self, label: MessageLabel, points: &[CompressedRistretto]) {
        self.append_message(label.as_bytes(), points_as_byte_slice(points));
    }

    #[tracing::instrument(
        name = "proofs.base.proof.transcript_protocol.challenge_ark_scalars",
        level = "info",
        skip_all
    )]
    fn challenge_ark_scalars(&mut self, scalars: &mut [ArkScalar], label: MessageLabel) {
        self.append_message(label.as_bytes(), &[]);
        struct TranscriptProtocolRng<'a>(&'a mut Transcript);
        impl<'a> ark_std::rand::RngCore for TranscriptProtocolRng<'a> {
            fn next_u32(&mut self) -> u32 {
                let mut buf = [0u8; 4];
                self.fill_bytes(&mut buf);
                u32::from_le_bytes(buf)
            }
            fn next_u64(&mut self) -> u64 {
                let mut buf = [0u8; 8];
                self.fill_bytes(&mut buf);
                u64::from_le_bytes(buf)
            }
            fn fill_bytes(&mut self, dest: &mut [u8]) {
                self.0.challenge_bytes(&[], dest);
            }
            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), ark_std::rand::Error> {
                self.fill_bytes(dest);
                Ok(())
            }
        }
        let rng = &mut TranscriptProtocolRng(self);
        for scalar in scalars.iter_mut() {
            *scalar = ArkScalar(ark_ff::UniformRand::rand(rng));
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
    QueryProof,
    QueryCommit,
    QueryMleEvaluations,
    QueryMleEvaluationsChallenge,
    QueryResultIndexes,
    QueryResultData,
    QuerySumcheckChallenge,
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
            MessageLabel::QueryProof => b"queryproof v1",
            MessageLabel::QueryCommit => b"querycommit v1",
            MessageLabel::QueryResultIndexes => b"queryresultindexes v1",
            MessageLabel::QueryResultData => b"queryresultdata v1",
            MessageLabel::QueryMleEvaluations => b"querymleevaluations v1",
            MessageLabel::QueryMleEvaluationsChallenge => b"querymleevaluationschallenge v1",
            MessageLabel::QuerySumcheckChallenge => b"querysumcheckchallenge v1",
        }
    }
}

fn points_as_byte_slice(slice: &[CompressedRistretto]) -> &[u8] {
    let slice = slice;
    let len = slice.len() * core::mem::size_of::<CompressedRistretto>();
    unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}
