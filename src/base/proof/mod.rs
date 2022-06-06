mod error;
pub use error::ProofError;

mod transcript;
#[cfg(test)]
mod transcript_test;
pub use transcript::TranscriptProtocol;

use curve25519_dalek::scalar::Scalar;

pub struct Commitment {
    //The actual commitment to a column/vector. It may make sense for this to be non compressed, and only serialized as compressed.
    pub commitment : curve25519_dalek::ristretto::CompressedRistretto,
    //The length of the column/vector.
    pub length : usize,
}

pub trait PIPProof {
    fn create(
        //The merlin transcript for the prover
        transcript: &mut dyn TranscriptProtocol, 
        //The inputs to the PIP. This is several columns. We may eventually wish for this to be a arrow::record_batch::RecordBatch instead.
        inputs : Vec<&[Scalar]>, 
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        outputs : Vec<&[Scalar]>,
    ) -> Self;
    fn verify(&self, 
        //The merlin transcript for the verifier
        transcript: &mut dyn TranscriptProtocol, 
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        inputs : Vec<Commitment>, 
        //The commitments of the outputs to the PIP. Typically, these are sent from the prover to the verifier before the PIPproof is created.
        outputs : Vec<Commitment>,
    ) -> Result<(), ProofError>;
    //fn to_bytes(&self) -> &[u8];
    //fn from_bytes(data : &[u8]) -> Self;
}