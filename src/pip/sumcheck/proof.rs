use ark_std::vec::Vec;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::{CompositePolynomial, CompositePolynomialInfo};
use crate::base::proof::ProofError;
use crate::base::proof::Transcript;
use crate::pip::sumcheck::prover_message::ProverMessage;

#[allow(dead_code)]
pub struct SumcheckProof {
    messages: Vec<ProverMessage>,
}

impl SumcheckProof {
    #[allow(unused_variables)]
    pub fn create(transcript: &mut Transcript, polynomial: &CompositePolynomial) -> SumcheckProof {
        transcript.sumcheck_domain_sep(
            polynomial.max_multiplicands as u64,
            polynomial.num_variables as u64,
        );
        let messages = Vec::with_capacity(0);
        SumcheckProof { messages: messages }
    }

    #[allow(unused_variables)]
    pub fn verify_without_evaluation(
        &self,
        evaluation_point: &mut [Scalar],
        transcript: &mut Transcript,
        polynomial_info: CompositePolynomialInfo,
    ) -> Result<(), ProofError> {
        transcript.sumcheck_domain_sep(
            polynomial_info.max_multiplicands as u64,
            polynomial_info.num_variables as u64,
        );
        Ok(())
    }
}
