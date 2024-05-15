use super::{G1Affine, G2Affine, F, GT};
use crate::base::{
    impl_serde_for_ark_serde_checked,
    proof::{MessageLabel, TranscriptProtocol},
};
use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use merlin::Transcript;
use num_traits::Zero;

#[derive(Default, Clone, CanonicalSerialize, CanonicalDeserialize, PartialEq, Eq, Debug)]
/// The messages sent from the prover to the verifier in the interactive protocol.
/// This is, in essence, the proof.
///
/// This struct is effectively 4 queues.
/// The prover pushes messages to the front of a queue, and the verifier pops messages from the back of a queue.
/// However, this functionality is hidden outside of `super`.
pub struct DoryMessages {
    /// The field elements sent from the prover to the verifier. The last element of the `Vec` is the first element sent.
    pub(super) F_messages: Vec<F>,
    /// The G1 elements sent from the prover to the verifier. The last element of the `Vec` is the first element sent.
    pub(super) G1_messages: Vec<G1Affine>,
    /// The G2 elements sent from the prover to the verifier. The last element of the `Vec` is the first element sent.
    pub(super) G2_messages: Vec<G2Affine>,
    /// The GT elements sent from the prover to the verifier. The last element of the `Vec` is the first element sent.
    pub(super) GT_messages: Vec<GT>,
}
impl_serde_for_ark_serde_checked!(DoryMessages);

#[cfg_attr(not(test), allow(dead_code))]
impl DoryMessages {
    /// Pushes a field element from the prover onto the queue, and appends it to the transcript.
    pub(super) fn prover_send_F_message(&mut self, transcript: &mut Transcript, message: F) {
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        self.F_messages.insert(0, message);
    }
    /// Pushes a G1 element from the prover onto the queue, and appends it to the transcript.
    pub(super) fn prover_send_G1_message(
        &mut self,
        transcript: &mut Transcript,
        message: impl Into<G1Affine>,
    ) {
        let message = message.into();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        self.G1_messages.insert(0, message);
    }
    /// Pushes a G2 element from the prover onto the queue, and appends it to the transcript.
    pub(super) fn prover_send_G2_message(
        &mut self,
        transcript: &mut Transcript,
        message: impl Into<G2Affine>,
    ) {
        let message = message.into();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        self.G2_messages.insert(0, message);
    }
    /// Pushes a GT element from the prover onto the queue, and appends it to the transcript.
    pub(super) fn prover_send_GT_message(&mut self, transcript: &mut Transcript, message: GT) {
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        self.GT_messages.insert(0, message);
    }
    /// Pops a field element from the verifier's queue, and appends it to the transcript.
    pub(super) fn prover_recieve_F_message(&mut self, transcript: &mut Transcript) -> F {
        let message = self.F_messages.pop().unwrap();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        message
    }
    /// Pops a G1 element from the verifier's queue, and appends it to the transcript.
    pub(super) fn prover_recieve_G1_message(&mut self, transcript: &mut Transcript) -> G1Affine {
        let message = self.G1_messages.pop().unwrap();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        message
    }
    /// Pops a G2 element from the verifier's queue, and appends it to the transcript.
    pub(super) fn prover_recieve_G2_message(&mut self, transcript: &mut Transcript) -> G2Affine {
        let message = self.G2_messages.pop().unwrap();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        message
    }
    /// Pops a GT element from the verifier's queue, and appends it to the transcript.
    pub(super) fn prover_recieve_GT_message(&mut self, transcript: &mut Transcript) -> GT {
        let message = self.GT_messages.pop().unwrap();
        transcript.append_canonical_serialize(MessageLabel::DoryMessage, &message);
        message
    }
    /// This is the F message that the verifier sends to the prover.
    /// This message is produces as a challenge from the transcript.
    ///
    /// While the message is a simple field element, we ensure that it is non-zero, and also return it's inverse.
    pub(super) fn verifier_F_message(&mut self, transcript: &mut Transcript) -> (F, F) {
        let mut message = F::zero();
        while message.is_zero() {
            transcript.challenge_ark(core::iter::once(&mut message), MessageLabel::DoryChallenge)
        }
        let message_inv = message.inverse().unwrap();
        (message, message_inv)
    }
}
