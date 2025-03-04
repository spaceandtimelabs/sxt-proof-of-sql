use super::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder};
use crate::base::{
    bit::BitDistribution,
    polynomial::MultilinearExtension,
    proof::ProofSizeMismatch,
    scalar::{test_scalar::TestScalar, Scalar},
};
use alloc::vec::Vec;
use core::iter;
use itertools::Itertools;

/// Track components used to verify a query's proof
pub struct MockVerificationBuilder<S: Scalar> {
    bit_distributions: Vec<BitDistribution>,
    bit_distribution_offset: usize,
    consumed_final_round_pcs_proof_mles: usize,
    subpolynomial_max_multiplicands: usize,

    evaluation_row_index: usize,
    final_round_mles: Vec<Vec<S>>,
    pub(crate) identity_subpolynomial_evaluations: Vec<Vec<S>>,
    pub(crate) zerosum_subpolynomial_evaluations: Vec<Vec<S>>,
}

impl<S: Scalar> VerificationBuilder<S> for MockVerificationBuilder<S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        match subpolynomial_type {
            SumcheckSubpolynomialType::Identity => {
                self.identity_subpolynomial_evaluations
                    .resize_with(self.evaluation_row_index + 1, Vec::new);
                if degree + 1 > self.subpolynomial_max_multiplicands {
                    Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                }
                self.identity_subpolynomial_evaluations[self.evaluation_row_index].push(eval);
            }
            SumcheckSubpolynomialType::ZeroSum => {
                self.zerosum_subpolynomial_evaluations
                    .resize_with(self.evaluation_row_index + 1, Vec::new);
                if degree > self.subpolynomial_max_multiplicands {
                    Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                }
                self.zerosum_subpolynomial_evaluations[self.evaluation_row_index].push(eval);
            }
        }
        Ok(())
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        let res = self
            .bit_distributions
            .get(self.bit_distribution_offset)
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewBitDistributions)?;
        self.bit_distribution_offset += 1;
        Ok(res)
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_final_round_pcs_proof_mles;
        self.consumed_final_round_pcs_proof_mles += 1;
        Ok(*self
            .final_round_mles
            .get(self.evaluation_row_index)
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)?
            .get(index)
            .unwrap_or(&S::ZERO))
    }

    fn singleton_chi_evaluation(&self) -> S {
        unimplemented!("No tests currently use this function")
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        iter::repeat_with(|| self.try_consume_final_round_mle_evaluation())
            .take(count)
            .collect()
    }
}

impl<S: Scalar> MockVerificationBuilder<S> {
    pub fn new(
        bit_distributions: Vec<BitDistribution>,
        subpolynomial_max_multiplicands: usize,
        final_round_mles: Vec<Vec<S>>,
    ) -> Self {
        Self {
            bit_distributions,
            bit_distribution_offset: 0,
            consumed_final_round_pcs_proof_mles: 0,
            subpolynomial_max_multiplicands,
            evaluation_row_index: 0,
            final_round_mles,
            identity_subpolynomial_evaluations: Vec::new(),
            zerosum_subpolynomial_evaluations: Vec::new(),
        }
    }

    pub fn increment_row_index(&mut self) {
        self.bit_distribution_offset = 0;
        self.consumed_final_round_pcs_proof_mles = 0;
        self.evaluation_row_index += 1;
    }

    pub fn get_identity_results(&self) -> Vec<Vec<bool>> {
        assert!(self
            .identity_subpolynomial_evaluations
            .iter()
            .map(Vec::len)
            .all_equal());
        self.identity_subpolynomial_evaluations
            .iter()
            .cloned()
            .map(|v| v.iter().map(S::is_zero).collect())
            .collect()
    }

    pub fn get_zero_sum_results(&self) -> Vec<bool> {
        assert!(self
            .zerosum_subpolynomial_evaluations
            .iter()
            .map(Vec::len)
            .all_equal());
        self.zerosum_subpolynomial_evaluations
            .iter()
            .cloned()
            .fold(
                vec![S::ZERO; self.zerosum_subpolynomial_evaluations.len()],
                |acc, row| {
                    acc.into_iter()
                        .zip(row)
                        .map(|(sum, val)| sum + val)
                        .collect()
                },
            )
            .iter()
            .map(S::is_zero)
            .collect()
    }
}

/// Allows testing `verify_evaluate` and other verify gadgets row by row.
/// The return matrix will tell which identity constraints failed.
/// Each vector represents the rseults of the constraints for each row.
/// The length of the vector should be the length of the data.
///
/// The return vector indicates the results of each constraint for the entire column
pub fn run_verify_for_each_row(
    table_length: usize,
    final_round_builder: &FinalRoundBuilder<'_, TestScalar>,
    subpolynomial_max_multiplicands: usize,
    row_verification: impl Fn(&mut MockVerificationBuilder<TestScalar>, TestScalar, &[TestScalar]),
) -> MockVerificationBuilder<TestScalar> {
    let evaluation_points: Vec<Vec<_>> = (0..table_length)
        .map(|i| {
            (0..table_length)
                .map(|j| {
                    if i == j {
                        TestScalar::ONE
                    } else {
                        TestScalar::ZERO
                    }
                })
                .collect()
        })
        .collect();
    let final_round_mles: Vec<_> = evaluation_points
        .iter()
        .map(|evaluation_point| final_round_builder.evaluate_pcs_proof_mles(evaluation_point))
        .collect();
    let mut verification_builder = MockVerificationBuilder::new(
        final_round_builder.bit_distributions().to_vec(),
        subpolynomial_max_multiplicands,
        final_round_mles,
    );

    for evaluation_point in evaluation_points {
        let one_eval =
            (&iter::repeat_n(1, table_length).collect::<Vec<_>>()).inner_product(&evaluation_point);
        row_verification(&mut verification_builder, one_eval, &evaluation_point);
        verification_builder.increment_row_index();
    }
    verification_builder
}

#[cfg(test)]
mod tests {
    use super::MockVerificationBuilder;
    use crate::{
        base::{
            bit::BitDistribution,
            proof::ProofSizeMismatch,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{SumcheckSubpolynomialType, VerificationBuilder},
    };

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_try_consume_rho_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder.try_consume_rho_evaluation().unwrap();
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_try_consume_first_round_mle_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_singleton_chi_evaluation() {
        let verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder.singleton_chi_evaluation();
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_try_consume_chi_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder.try_consume_chi_evaluation().unwrap();
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_rho_256_evaluation() {
        let verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder.rho_256_evaluation().unwrap();
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_try_consume_post_result_challenge() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        verification_builder
            .try_consume_post_result_challenge()
            .unwrap();
    }

    #[test]
    fn we_can_get_sumcheck_proof_too_small_for_identity() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        let error = verification_builder
            .try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                TestScalar::ONE,
                2,
            )
            .unwrap_err();
        assert!(matches!(error, ProofSizeMismatch::SumcheckProofTooSmall));
    }

    #[test]
    fn we_can_get_sumcheck_proof_too_small_for_zero_sum() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        let error = verification_builder
            .try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::ZeroSum,
                TestScalar::ONE,
                3,
            )
            .unwrap_err();
        assert!(matches!(error, ProofSizeMismatch::SumcheckProofTooSmall));
    }

    #[test]
    fn we_can_get_final_round_mle_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                vec![
                    vec![TestScalar::ONE, TestScalar::TWO],
                    vec![-TestScalar::ONE, -TestScalar::TWO],
                ],
            );
        let result = verification_builder
            .try_consume_final_round_mle_evaluations(2)
            .unwrap();
        assert_eq!(result, vec![TestScalar::ONE, TestScalar::TWO]);
    }

    #[test]
    fn we_can_get_error_when_not_enough_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        let error = verification_builder
            .try_consume_final_round_mle_evaluations(2)
            .unwrap_err();
        assert!(matches!(error, ProofSizeMismatch::TooFewMLEEvaluations));
    }

    #[test]
    fn we_can_try_consume_bit_distribution() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                vec![BitDistribution::new::<TestScalar, TestScalar>(&[
                    TestScalar::ONE,
                ])],
                2,
                Vec::new(),
            );
        let result = verification_builder.try_consume_bit_distribution().unwrap();
        assert_eq!(
            result,
            BitDistribution::new::<TestScalar, TestScalar>(&[TestScalar::ONE])
        );
    }

    #[test]
    fn we_can_get_error_when_not_enough_bit_distributions() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(Vec::new(), 2, Vec::new());
        let error = verification_builder
            .try_consume_bit_distribution()
            .unwrap_err();
        assert!(matches!(error, ProofSizeMismatch::TooFewBitDistributions));
    }
}
