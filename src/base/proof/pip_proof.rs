use curve25519_dalek::scalar::Scalar;

use crate::base::proof::{Commitment, ProofError, Transcript};

pub trait PIPProof {
    fn create(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP. This is several columns. We may eventually wish for this to be a arrow::record_batch::RecordBatch instead.
        inputs: Vec<&[Scalar]>,
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        outputs: Vec<&[Scalar]>,
    ) -> Self;
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        inputs: Vec<Commitment>,
        //The commitments of the outputs to the PIP. Typically, these are sent from the prover to the verifier before the PIPProof is created.
        outputs: Vec<Commitment>,
    ) -> Result<(), ProofError>;
    //fn to_bytes(&self) -> &[u8];
    //fn from_bytes(data : &[u8]) -> Self;
}
