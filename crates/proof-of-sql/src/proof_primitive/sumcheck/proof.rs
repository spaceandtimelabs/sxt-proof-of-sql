use crate::{
    base::{
        polynomial::{CompositePolynomial, CompositePolynomialInfo},
        proof::{ProofError, Transcript},
        scalar::Scalar,
    },
    proof_primitive::sumcheck::{prove_round, ProverState, Subclaim},
};
use serde::{Deserialize, Serialize};
/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use std::vec::Vec;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SumcheckProof<S: Scalar> {
    pub(super) evaluations: Vec<Vec<S>>,
}

impl<S: Scalar> SumcheckProof<S> {
    #[tracing::instrument(name = "SumcheckProof::create", level = "debug", skip_all)]
    pub fn create(
        transcript: &mut impl Transcript,
        evaluation_point: &mut [S],
        polynomial: &CompositePolynomial<S>,
    ) -> Self {
        assert_eq!(evaluation_point.len(), polynomial.num_variables);
        transcript.extend_as_be([
            polynomial.max_multiplicands as u64,
            polynomial.num_variables as u64,
        ]);
        // This challenge is in order to keep transcript messages grouped. (This simplifies the Solidity implementation.)
        transcript.scalar_challenge_as_be::<S>();
        let mut r = None;
        let mut state = ProverState::create(polynomial);
        let mut evaluations = Vec::with_capacity(polynomial.num_variables);
        for scalar in evaluation_point.iter_mut().take(polynomial.num_variables) {
            let round_evaluations = prove_round(&mut state, &r);
            transcript.extend_scalars_as_be(&round_evaluations);
            *scalar = transcript.scalar_challenge_as_be();
            evaluations.push(round_evaluations);
            r = Some(*scalar);
        }

        SumcheckProof { evaluations }
    }

    #[tracing::instrument(
        name = "SumcheckProof::verify_without_evaluation",
        level = "debug",
        skip_all
    )]
    pub fn verify_without_evaluation(
        &self,
        transcript: &mut impl Transcript,
        polynomial_info: CompositePolynomialInfo,
        claimed_sum: &S,
    ) -> Result<Subclaim<S>, ProofError> {
        transcript.extend_as_be([
            polynomial_info.max_multiplicands as u64,
            polynomial_info.num_variables as u64,
        ]);
        // This challenge is in order to keep transcript messages grouped. (This simplifies the Solidity implementation.)
        transcript.scalar_challenge_as_be::<S>();
        if self.evaluations.len() != polynomial_info.num_variables {
            return Err(ProofError::VerificationError(
                "invalid number of evaluations",
            ));
        }
        let mut evaluation_point = Vec::with_capacity(polynomial_info.num_variables);
        for round_index in 0..polynomial_info.num_variables {
            transcript.extend_scalars_as_be(&self.evaluations[round_index]);
            evaluation_point.push(transcript.scalar_challenge_as_be());
        }
        Subclaim::create(
            evaluation_point,
            &self.evaluations,
            polynomial_info.max_multiplicands,
            claimed_sum,
        )
    }
}
