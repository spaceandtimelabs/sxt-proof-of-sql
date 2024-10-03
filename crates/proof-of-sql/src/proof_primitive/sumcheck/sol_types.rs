use super::{proof::Subclaim, SumcheckProof};
use crate::base::scalar::Scalar;
use alloy_sol_types::{private::primitives::U256, sol};

sol!("./sol_src/proof_primitive/sumcheck/Sumcheck.sol");

impl<S: Scalar> From<SumcheckProof<S>> for Sumcheck::Proof {
    fn from(value: SumcheckProof<S>) -> Self {
        Self {
            coefficients: value
                .coefficients
                .into_iter()
                .map(Into::into)
                .map(U256::from_limbs)
                .collect(),
        }
    }
}
impl<S: Scalar> From<Subclaim<S>> for Sumcheck::Subclaim {
    fn from(value: Subclaim<S>) -> Self {
        Self {
            evaluationPoint: value
                .evaluation_point
                .into_iter()
                .map(Into::into)
                .map(U256::from_limbs)
                .collect(),
            expectedEvaluation: U256::from_limbs(value.expected_evaluation.into()),
        }
    }
}
