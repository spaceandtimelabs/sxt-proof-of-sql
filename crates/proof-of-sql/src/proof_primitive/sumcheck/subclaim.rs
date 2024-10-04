/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::base::scalar::Scalar;
use crate::base::{polynomial::interpolate_uni_poly, proof::ProofError};
use alloc::vec::Vec;

pub struct Subclaim<S: Scalar> {
    pub evaluation_point: Vec<S>,
    pub expected_evaluation: S,
}

impl<S: Scalar> Subclaim<S> {
    /// # Panics
    /// Panics if `max_multiplicands` is less than or equal to 0.
    /// Panics if the length of `evaluation_point` does not match the length of `evaluations`.
    /// Panics if any round evaluation does not match the expected length of `max_multiplicands + 1`.
    /// Panics if the claimed sum does not match the sum of the first two evaluations in the round.
    pub(super) fn create(
        evaluation_point: Vec<S>,
        evaluations: &[Vec<S>],
        max_multiplicands: usize,
        claimed_sum: &S,
    ) -> Result<Subclaim<S>, ProofError> {
        let num_vars = evaluation_point.len();
        assert!(max_multiplicands > 0);
        assert_eq!(num_vars, evaluations.len());
        let mut expected_sum = *claimed_sum;
        for round_index in 0..num_vars {
            let round_evaluation = &evaluations[round_index];
            if round_evaluation.len() != max_multiplicands + 1 {
                return Err(ProofError::VerificationError {
                    error: "round evaluation length does not match max multiplicands",
                });
            }
            if expected_sum != round_evaluation[0] + round_evaluation[1] {
                return Err(ProofError::VerificationError {
                    error: "round evaluation does not match claimed sum",
                });
            }
            expected_sum = interpolate_uni_poly(round_evaluation, evaluation_point[round_index]);
        }
        Ok(Subclaim {
            evaluation_point,
            expected_evaluation: expected_sum,
        })
    }
}
