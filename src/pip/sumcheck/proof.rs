use ark_std::vec::Vec;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::{CompositePolynomial, CompositePolynomialInfo};
use crate::base::proof::{ProofError, Transcript};
use crate::pip::sumcheck::{prove_round, ProverState, Subclaim};

#[allow(dead_code)]
pub struct SumcheckProof {
    pub evaluations: Vec<Vec<Scalar>>,
}

impl SumcheckProof {
    pub fn create(
        evaluation_point: &mut [Scalar],
        transcript: &mut Transcript,
        polynomial: &CompositePolynomial,
    ) -> SumcheckProof {
        assert_eq!(evaluation_point.len(), polynomial.num_variables);
        transcript.sumcheck_domain_sep(
            polynomial.max_multiplicands as u64,
            polynomial.num_variables as u64,
        );
        let mut r = None;
        let mut state = ProverState::create(&polynomial);
        let mut evaluations = Vec::with_capacity(polynomial.num_variables);
        for round_index in 0..polynomial.num_variables {
            let round_evaluations = prove_round(&mut state, &r);
            transcript.append_scalars(b"P", &round_evaluations);
            evaluations.push(round_evaluations);
            evaluation_point[round_index] = transcript.challenge_scalar(b"r");
            r = Some(evaluation_point[round_index]);
        }

        SumcheckProof {
            evaluations: evaluations,
        }
    }

    #[allow(unused_variables)]
    pub fn verify_without_evaluation(
        &self,
        transcript: &mut Transcript,
        polynomial_info: CompositePolynomialInfo,
        claimed_sum: &Scalar,
    ) -> Result<Subclaim, ProofError> {
        transcript.sumcheck_domain_sep(
            polynomial_info.max_multiplicands as u64,
            polynomial_info.num_variables as u64,
        );
        if self.evaluations.len() != polynomial_info.num_variables {
            return Err(ProofError::VerificationError);
        }
        let mut evaluation_point = Vec::with_capacity(polynomial_info.num_variables);
        for round_index in 0..polynomial_info.num_variables {
            transcript.append_scalars(b"P", &self.evaluations[round_index]);
            evaluation_point.push(transcript.challenge_scalar(b"r"));
        }
        Subclaim::create(
            evaluation_point,
            &self.evaluations,
            polynomial_info.max_multiplicands,
            claimed_sum,
        )
    }
}
