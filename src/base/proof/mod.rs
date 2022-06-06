mod error;
pub use error::ProofError;

mod transcript;
#[cfg(test)]
mod transcript_test;
pub use transcript::TranscriptProtocol;

use curve25519_dalek::scalar::Scalar;

pub struct Commitment {
    pub commitment : curve25519_dalek::ristretto::CompressedRistretto,
    pub length : usize,
}

pub trait PIPProof {
    fn create(
        transcript: &mut dyn TranscriptProtocol, 
        inputs : Vec<&[Scalar]>, 
        outputs : Vec<&[Scalar]>,
    ) -> Self;
    fn verify(&self, 
        transcript: &mut dyn TranscriptProtocol, 
        inputs : Vec<Commitment>, 
        outputs : Vec<Commitment>,
    ) -> Result<(), ProofError>;
}