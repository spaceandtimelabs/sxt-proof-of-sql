use ark_ff::Field;
use ark_std::vec::Vec;
use merlin::Transcript;

pub mod prover_message;
pub mod polynomial;
use crate::errors::ProofError;
use crate::pip::sumcheck::prover_message::ProverMessage;
use crate::pip::sumcheck::polynomial::Polynomial;

// #[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SumcheckProof<F: Field> {
    messages: Vec<ProverMessage<F>>,
}

impl<F: Field> SumcheckProof<F> {
    #[allow(unused_variables)]
    pub fn create(
        transcript: &mut Transcript,
        polynomial: &Polynomial<F>,
    ) -> SumcheckProof<F>{
        let messages = Vec::with_capacity(0);
        SumcheckProof{
            messages: messages,
        }
    }

    #[allow(unused_variables)]
    pub fn verify_without_evaluation(
        &self,
        transcript: &mut Transcript,
    ) -> Result<(), ProofError> {
        Ok(())
    }

}
