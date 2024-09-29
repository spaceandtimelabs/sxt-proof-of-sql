use ark_serialize::CanonicalSerialize;
use merlin::Transcript;

/// Merlin Transcripts, for non-interactive proofs.
///
/// Think of this as a transaction log.
/// When adding data to it, the opaque state will change in a pseudorandom way.
/// The provers and verifiers should be able to add the same data in the same order, and achieve the same state.
///
/// In most cases it looks like this:
/// ```ignore
/// use proof_of_sql::base::proof::{Commitment, MessageLabel, transcript::Transcript};
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
    ///
    /// The message is encoded with Postcard v1, chosen for its simplicity and stability.
    fn append_auto(&mut self, label: MessageLabel, message: &(impl serde::Serialize + ?Sized));

    /// Append a message to the transcript, serializing it with CanonicalSerialize.
    fn append_canonical_serialize(
        &mut self,
        label: MessageLabel,
        message: &(impl CanonicalSerialize + ?Sized),
    );

    /// Compute multiple challenge values of a type that extends `ark_std::UniformRand` (which requires a label). This generalizes `challenge_curve25519_scalars`.
    fn challenge_scalars<'a, U: ark_std::UniformRand + 'a>(
        &mut self,
        buf: impl IntoIterator<Item = &'a mut U>,
        label: MessageLabel,
    );

    /// Compute a challenge variable (which requires a label).
    #[cfg(test)]
    fn challenge_scalar_single<U: ark_std::UniformRand + Default>(
        &mut self,
        label: MessageLabel,
    ) -> U {
        let mut res = Default::default();
        self.challenge_scalars(core::iter::once(&mut res), label);
        res
    }
}

impl TranscriptProtocol for Transcript {
    /// # Panics
    /// - Panics if `postcard::to_allocvec(message)` fails to serialize the message.
    fn append_auto(&mut self, label: MessageLabel, message: &(impl serde::Serialize + ?Sized)) {
        self.append_message(label.as_bytes(), &postcard::to_allocvec(message).unwrap());
    }

    /// # Panics
    /// - Panics if `message.serialize_compressed(&mut buf)` fails to serialize the message.
    fn append_canonical_serialize(
        &mut self,
        label: MessageLabel,
        message: &(impl CanonicalSerialize + ?Sized),
    ) {
        let mut buf = vec![Default::default(); message.compressed_size()];
        message.serialize_compressed(&mut buf).unwrap();
        self.append_message(label.as_bytes(), &buf);
    }

    fn challenge_scalars<'a, U: ark_std::UniformRand + 'a>(
        &mut self,
        buf: impl IntoIterator<Item = &'a mut U>,
        label: MessageLabel,
    ) {
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
        for val in buf {
            *val = ark_ff::UniformRand::rand(rng);
        }
    }
}

/// Labels for items in a merlin transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLabel {
    /// Represents an inner product computation or its result.
    #[cfg(test)]
    InnerProduct,
    /// Represents a challenge in the computation of an inner product.
    #[cfg(test)]
    InnerProductChallenge,
    /// Represents a challenge in the sumcheck protocol.
    #[cfg(test)]
    SumcheckChallenge,
    /// Represents a proof resulting from a query.
    QueryProof,
    /// Represents a commitment to a query.
    QueryCommit,
    /// Represents evaluations in an MLE context.
    QueryMleEvaluations,
    /// Represents a challenge in the context of MLE evaluations.
    QueryMleEvaluationsChallenge,
    /// Represents the data resulting from a query.
    QueryResultData,
    /// Represents a query for bit distribution data.
    QueryBitDistributions,
    /// Represents a challenge in a sumcheck query.
    QuerySumcheckChallenge,
    /// Represents a hash used for verification purposes.
    VerificationHash,
    /// Represents a message in the context of the Dory protocol.
    DoryMessage,
    /// Represents a challenge in the context of the Dory protocol.
    DoryChallenge,
    /// Represents challenges posted after result computation.
    PostResultChallenges,
    /// Represents a SQL query
    ProofPlan,
    /// Represents the length of a table.
    TableLength,
    /// Represents an offset for a generator.
    GeneratorOffset,
}

impl MessageLabel {
    /// Convert the label to a byte slice, which satisfies the requirements of a merlin label:
    /// "the labels should be distinct and none should be a prefix of any other."
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            #[cfg(test)]
            MessageLabel::InnerProduct => b"ipp v1",
            #[cfg(test)]
            MessageLabel::InnerProductChallenge => b"ippchallenge v1",
            #[cfg(test)]
            MessageLabel::SumcheckChallenge => b"sumcheckchallenge v1",
            MessageLabel::QueryProof => b"queryproof v1",
            MessageLabel::QueryCommit => b"querycommit v1",
            MessageLabel::QueryResultData => b"queryresultdata v1",
            MessageLabel::QueryBitDistributions => b"querybitdistributions v1",
            MessageLabel::QueryMleEvaluations => b"querymleevaluations v1",
            MessageLabel::QueryMleEvaluationsChallenge => b"querymleevaluationschallenge v1",
            MessageLabel::QuerySumcheckChallenge => b"querysumcheckchallenge v1",
            MessageLabel::VerificationHash => b"verificationhash v1",
            MessageLabel::DoryMessage => b"dorymessage v1",
            MessageLabel::DoryChallenge => b"dorychallenge v1",
            MessageLabel::PostResultChallenges => b"postresultchallenges v1",
            MessageLabel::ProofPlan => b"proofplan v1",
            MessageLabel::TableLength => b"tablelength v1",
            MessageLabel::GeneratorOffset => b"generatoroffset v1",
        }
    }
}

impl super::transcript_core::TranscriptCore for merlin::Transcript {
    fn new() -> Self {
        merlin::Transcript::new(b"TranscriptCore::new")
    }
    fn raw_append(&mut self, message: &[u8]) {
        self.append_message(b"TranscriptCore::raw_append", message)
    }
    fn raw_challenge(&mut self) -> [u8; 32] {
        let mut result = [0u8; 32];
        self.challenge_bytes(b"TranscriptCore::raw_challenge", &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::transcript_core::test_util::*;
    #[test]
    fn we_get_equivalent_challenges_with_equivalent_merlin_transcripts() {
        we_get_equivalent_challenges_with_equivalent_transcripts::<merlin::Transcript>()
    }
    #[test]
    fn we_get_different_challenges_with_different_keccak256_transcripts() {
        we_get_different_challenges_with_different_transcripts::<merlin::Transcript>()
    }
    #[test]
    fn we_get_different_nontrivial_consecutive_challenges_from_keccak256_transcript() {
        we_get_different_nontrivial_consecutive_challenges_from_transcript::<merlin::Transcript>()
    }
}
