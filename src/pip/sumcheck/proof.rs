use ark_std::vec::Vec;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::{CompositePolynomial, CompositePolynomialInfo};
use crate::base::proof::{ProofError, Transcript};
use crate::pip::sumcheck::{prove_round, ProverState};

#[allow(dead_code)]
pub struct SumcheckProof {
    evaluations: Vec<Vec<Scalar>>,
}

impl SumcheckProof {
    pub fn create(transcript: &mut Transcript, polynomial: &CompositePolynomial) -> SumcheckProof {
        transcript.sumcheck_domain_sep(
            polynomial.max_multiplicands as u64,
            polynomial.num_variables as u64,
        );
        let mut r = None;
        let mut state = ProverState::create(&polynomial);
        let mut evaluations = Vec::with_capacity(polynomial.num_variables);
        for _ in 0..polynomial.num_variables {
            let round_evaluations = prove_round(&mut state, &r);
            transcript.append_scalars(b"P", &round_evaluations);
            evaluations.push(round_evaluations);
            r = Some(transcript.challenge_scalar(b"r"));
        }

        SumcheckProof {
            evaluations: evaluations,
        }
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
