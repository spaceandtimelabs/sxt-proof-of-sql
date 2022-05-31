use ark_std::vec::Vec;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::base::polynomial::CompositePolynomial;
use crate::base::proof::ProofError;
use crate::pip::sumcheck::prover_message::ProverMessage;

#[allow(dead_code)]
pub struct SumcheckProof {
    messages: Vec<ProverMessage>,
}

impl SumcheckProof {
    #[allow(unused_variables)]
    pub fn create(transcript: &mut Transcript, polynomial: &CompositePolynomial) -> SumcheckProof {
        let messages = Vec::with_capacity(0);
        SumcheckProof { messages: messages }
    }

    #[allow(unused_variables)]
    pub fn verify_without_evaluation(
        &self,
        evaluation_point: &mut [Scalar],
        transcript: &mut Transcript,
    ) -> Result<(), ProofError> {
        Ok(())
    }
}
