use crate::{
    base::{
        polynomial::{CompositePolynomial, CompositePolynomialInfo},
        proof::{MessageLabel, ProofError, TranscriptProtocol},
        scalar::Scalar,
    },
    proof_primitive::sumcheck::{prove_round, ProverState, Subclaim},
};
use merlin::Transcript;
use serde::{Deserialize, Serialize};
/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use std::vec::Vec;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// TODO: add docs
pub struct SumcheckProof<S: Scalar> {
    pub(super) evaluations: Vec<Vec<S>>,
}

impl<S: Scalar> SumcheckProof<S> {
    #[tracing::instrument(
        name = "proofs.proof_primitive.sumcheck.proof.create",
        level = "info",
        skip_all
    )]
    /// TODO: add docs
    pub fn create(
        transcript: &mut Transcript,
        evaluation_point: &mut [S],
        polynomial: &CompositePolynomial<S>,
    ) -> Self {
        assert_eq!(evaluation_point.len(), polynomial.num_variables);
        transcript.append_auto(
            MessageLabel::Sumcheck,
            &(polynomial.max_multiplicands, polynomial.num_variables),
        );
        let mut r = None;
        let mut state = ProverState::create(polynomial);
        let mut evaluations = Vec::with_capacity(polynomial.num_variables);
        for scalar in evaluation_point.iter_mut().take(polynomial.num_variables) {
            let round_evaluations = prove_round(&mut state, &r);
            transcript.append_canonical_serialize(
                MessageLabel::SumcheckRoundEvaluation,
                &round_evaluations,
            );
            evaluations.push(round_evaluations);
            *scalar = transcript.challenge_curve25519_single(MessageLabel::SumcheckChallenge);
            r = Some(*scalar);
        }

        SumcheckProof { evaluations }
    }

    #[tracing::instrument(
        name = "proofs.proof_primitive.sumcheck.proof.verify_without_evaluation",
        level = "debug",
        skip_all
    )]
    /// TODO: add docs
    pub fn verify_without_evaluation(
        &self,
        transcript: &mut Transcript,
        polynomial_info: CompositePolynomialInfo,
        claimed_sum: &S,
    ) -> Result<Subclaim<S>, ProofError> {
        transcript.append_auto(
            MessageLabel::Sumcheck,
            &(
                polynomial_info.max_multiplicands,
                polynomial_info.num_variables,
            ),
        );
        if self.evaluations.len() != polynomial_info.num_variables {
            return Err(ProofError::VerificationError(
                "invalid number of evaluations",
            ));
        }
        let mut evaluation_point = Vec::with_capacity(polynomial_info.num_variables);
        for round_index in 0..polynomial_info.num_variables {
            transcript.append_canonical_serialize(
                MessageLabel::SumcheckRoundEvaluation,
                &self.evaluations[round_index],
            );
            evaluation_point
                .push(transcript.challenge_curve25519_single(MessageLabel::SumcheckChallenge));
        }
        Subclaim::create(
            evaluation_point,
            &self.evaluations,
            polynomial_info.max_multiplicands,
            claimed_sum,
        )
    }
}
