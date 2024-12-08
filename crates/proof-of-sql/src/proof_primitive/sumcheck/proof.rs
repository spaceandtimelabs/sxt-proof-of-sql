use crate::{
    base::{
        polynomial::{interpolate_evaluations_to_reverse_coefficients, CompositePolynomialInfo},
        proof::{ProofError, Transcript},
        scalar::Scalar,
    },
    proof_primitive::sumcheck::{prove_round, ProverState},
};
/*
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SumcheckProof<S: Scalar> {
    pub(super) coefficients: Vec<S>,
}
pub struct Subclaim<S: Scalar> {
    pub evaluation_point: Vec<S>,
    pub expected_evaluation: S,
}

impl<S: Scalar> SumcheckProof<S> {
    #[tracing::instrument(name = "SumcheckProof::create", level = "debug", skip_all)]
    pub fn create(
        transcript: &mut impl Transcript,
        evaluation_point: &mut [S],
        mut state: ProverState<S>,
    ) -> Self {
        let num_vars = state.num_vars;
        assert_eq!(evaluation_point.len(), num_vars);
        transcript.extend_as_be([state.max_multiplicands as u64, num_vars as u64]);
        // This challenge is in order to keep transcript messages grouped. (This simplifies the Solidity implementation.)
        transcript.scalar_challenge_as_be::<S>();
        let mut r = None;
        let mut coefficients = Vec::with_capacity(num_vars);
        for scalar in evaluation_point.iter_mut().take(num_vars) {
            let round_evaluations = prove_round(&mut state, &r);
            let round_coefficients =
                interpolate_evaluations_to_reverse_coefficients(&round_evaluations);
            transcript.extend_scalars_as_be(&round_coefficients);
            coefficients.extend(round_coefficients);
            *scalar = transcript.scalar_challenge_as_be();
            r = Some(*scalar);
        }

        SumcheckProof { coefficients }
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
        if self.coefficients.len()
            != polynomial_info.num_variables * (polynomial_info.max_multiplicands + 1)
        {
            return Err(ProofError::VerificationError {
                error: "invalid proof size",
            });
        }
        let mut evaluation_point = Vec::with_capacity(polynomial_info.num_variables);

        let mut expected_evaluation = *claimed_sum;
        for round_index in 0..polynomial_info.num_variables {
            let start_index = round_index * (polynomial_info.max_multiplicands + 1);
            transcript.extend_scalars_as_be(
                &self.coefficients[start_index..=(start_index + polynomial_info.max_multiplicands)],
            );
            let round_evaluation_point = transcript.scalar_challenge_as_be();
            evaluation_point.push(round_evaluation_point);
            let mut round_evaluation = self.coefficients[start_index];
            let mut actual_sum = round_evaluation
                + self.coefficients[start_index + polynomial_info.max_multiplicands];
            for coefficient_index in
                (start_index + 1)..=(start_index + polynomial_info.max_multiplicands)
            {
                round_evaluation *= round_evaluation_point;
                round_evaluation += self.coefficients[coefficient_index];
                actual_sum += self.coefficients[coefficient_index];
            }
            if actual_sum != expected_evaluation {
                return Err(ProofError::VerificationError {
                    error: "round evaluation does not match claimed sum",
                });
            }
            expected_evaluation = round_evaluation;
        }
        Ok(Subclaim {
            evaluation_point,
            expected_evaluation,
        })
    }
}
