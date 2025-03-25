use super::{FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder};
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
    consumed_first_round_pcs_proof_mles: usize,
    consumed_final_round_pcs_proof_mles: usize,
    consumed_chi_evaluations: usize,
    consumed_rho_evaluations: usize,
    subpolynomial_max_multiplicands: usize,

    evaluation_row_index: usize,
    first_round_mles: Vec<Vec<S>>,
    final_round_mles: Vec<Vec<S>>,
    chi_evaluation_length_queue: Vec<usize>,
    rho_evaluation_length_queue: Vec<usize>,
    pub(crate) identity_subpolynomial_evaluations: Vec<Vec<S>>,
    pub(crate) zerosum_subpolynomial_evaluations: Vec<Vec<S>>,
}

impl<S: Scalar> VerificationBuilder<S> for MockVerificationBuilder<S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let length = self
            .chi_evaluation_length_queue
            .get(self.consumed_chi_evaluations)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewChiLengths)?;
        self.consumed_chi_evaluations += 1;
        Ok(if self.evaluation_row_index < length {
            S::ONE
        } else {
            S::ZERO
        })
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

    fn try_consume_byte_distribution(
        &mut self,
    ) -> Result<crate::base::byte::ByteDistribution, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let length = self
            .rho_evaluation_length_queue
            .get(self.consumed_rho_evaluations)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewRhoLengths)?;
        self.consumed_rho_evaluations += 1;
        Ok(if self.evaluation_row_index < length {
            S::from(self.evaluation_row_index as u64)
        } else {
            S::ZERO
        })
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_first_round_pcs_proof_mles;
        self.consumed_first_round_pcs_proof_mles += 1;
        self.first_round_mles
            .get(self.evaluation_row_index)
            .cloned()
            .map_or(Ok(S::ZERO), |v| {
                v.get(index)
                    .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
                    .copied()
            })
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_final_round_pcs_proof_mles;
        self.consumed_final_round_pcs_proof_mles += 1;
        self.final_round_mles
            .get(self.evaluation_row_index)
            .cloned()
            .map_or(Ok(S::ZERO), |v| {
                v.get(index)
                    .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
                    .copied()
            })
    }

    fn singleton_chi_evaluation(&self) -> S {
        unimplemented!("No tests currently use this function")
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        Some(if self.evaluation_row_index < 256 {
            S::from(u8::try_from(self.evaluation_row_index).unwrap())
        } else {
            S::ZERO
        })
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
        first_round_mles: Vec<Vec<S>>,
        final_round_mles: Vec<Vec<S>>,
        chi_evaluation_length_queue: Vec<usize>,
        rho_evaluation_length_queue: Vec<usize>,
    ) -> Self {
        Self {
            bit_distributions,
            bit_distribution_offset: 0,
            consumed_first_round_pcs_proof_mles: 0,
            consumed_final_round_pcs_proof_mles: 0,
            consumed_chi_evaluations: 0,
            consumed_rho_evaluations: 0,
            subpolynomial_max_multiplicands,
            evaluation_row_index: 0,
            first_round_mles,
            final_round_mles,
            chi_evaluation_length_queue,
            rho_evaluation_length_queue,
            identity_subpolynomial_evaluations: Vec::new(),
            zerosum_subpolynomial_evaluations: Vec::new(),
        }
    }

    pub fn increment_row_index(&mut self) {
        self.bit_distribution_offset = 0;
        self.consumed_final_round_pcs_proof_mles = 0;
        self.consumed_chi_evaluations = 0;
        self.consumed_first_round_pcs_proof_mles = 0;
        self.consumed_rho_evaluations = 0;
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
    first_round_builder: &FirstRoundBuilder<'_, TestScalar>,
    final_round_builder: &FinalRoundBuilder<'_, TestScalar>,
    subpolynomial_max_multiplicands: usize,
    row_verification: impl Fn(&mut MockVerificationBuilder<TestScalar>, TestScalar, &[TestScalar]),
) -> MockVerificationBuilder<TestScalar> {
    let evaluation_points: Vec<Vec<_>> = (0..first_round_builder.range_length())
        .map(|i| {
            (0..first_round_builder.range_length())
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
    let first_round_mles: Vec<_> = evaluation_points
        .iter()
        .map(|evaluation_point| first_round_builder.evaluate_pcs_proof_mles(evaluation_point))
        .collect();
    let final_round_mles: Vec<_> = evaluation_points
        .iter()
        .map(|evaluation_point| final_round_builder.evaluate_pcs_proof_mles(evaluation_point))
        .collect();
    let mut verification_builder = MockVerificationBuilder::new(
        final_round_builder.bit_distributions().to_vec(),
        subpolynomial_max_multiplicands,
        first_round_mles,
        final_round_mles,
        first_round_builder.chi_evaluation_lengths().to_vec(),
        first_round_builder.rho_evaluation_lengths().to_vec(),
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

    #[test]
    fn we_can_try_consume_rho_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![3],
            );
        let zero = verification_builder.try_consume_rho_evaluation().unwrap();
        assert_eq!(zero, TestScalar::ZERO);
        verification_builder.increment_row_index();
        let one = verification_builder.try_consume_rho_evaluation().unwrap();
        assert_eq!(one, TestScalar::ONE);
        verification_builder.increment_row_index();
        let two = verification_builder.try_consume_rho_evaluation().unwrap();
        assert_eq!(two, TestScalar::TWO);
        verification_builder.increment_row_index();
        let zero = verification_builder.try_consume_rho_evaluation().unwrap();
        assert_eq!(zero, TestScalar::ZERO);
    }

    #[test]
    fn we_can_get_error_if_try_consume_rho_evaluation_with_too_few_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![3],
            );
        let zero = verification_builder.try_consume_rho_evaluation().unwrap();
        assert_eq!(zero, TestScalar::ZERO);
        let err = verification_builder
            .try_consume_rho_evaluation()
            .unwrap_err();
        assert!(matches!(err, ProofSizeMismatch::TooFewRhoLengths));
    }

    #[test]
    fn we_can_try_consume_first_round_mle_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                vec![
                    vec![TestScalar::ONE, TestScalar::TWO],
                    vec![-TestScalar::ONE, -TestScalar::TWO],
                ],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        let one = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
        assert_eq!(one, TestScalar::ONE);
        let two = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
        assert_eq!(two, TestScalar::TWO);
        verification_builder.increment_row_index();
        let negative_one = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
        assert_eq!(negative_one, -TestScalar::ONE);
        let negative_two = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
        assert_eq!(negative_two, -TestScalar::TWO);
    }

    #[test]
    fn we_can_get_error_if_try_consume_first_round_mle_evaluation_with_too_few_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                vec![vec![TestScalar::ONE]],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        let one = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap();
        assert_eq!(one, TestScalar::ONE);
        let err = verification_builder
            .try_consume_first_round_mle_evaluation()
            .unwrap_err();
        assert!(matches!(err, ProofSizeMismatch::TooFewMLEEvaluations));
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_singleton_chi_evaluation() {
        let verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        verification_builder.singleton_chi_evaluation();
    }

    #[test]
    fn we_can_try_consume_chi_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                vec![2],
                Vec::new(),
            );
        let one = verification_builder.try_consume_chi_evaluation().unwrap();
        assert_eq!(one, TestScalar::ONE);
        verification_builder.increment_row_index();
        let one = verification_builder.try_consume_chi_evaluation().unwrap();
        assert_eq!(one, TestScalar::ONE);
        verification_builder.increment_row_index();
        let zero = verification_builder.try_consume_chi_evaluation().unwrap();
        assert_eq!(zero, TestScalar::ZERO);
    }

    #[test]
    fn we_can_get_error_if_try_consume_chi_evaluation_with_too_few_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                vec![3],
                Vec::new(),
            );
        let one = verification_builder.try_consume_chi_evaluation().unwrap();
        assert_eq!(one, TestScalar::ONE);
        let err = verification_builder
            .try_consume_chi_evaluation()
            .unwrap_err();
        assert!(matches!(err, ProofSizeMismatch::TooFewChiLengths));
    }

    #[test]
    fn we_can_get_rho_256_evaluation() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        for i in 0..256 {
            let val = verification_builder.rho_256_evaluation().unwrap();
            assert_eq!(val, TestScalar::from(i));
            verification_builder.increment_row_index();
        }
        let val = verification_builder.rho_256_evaluation().unwrap();
        assert_eq!(val, TestScalar::ZERO);
    }

    #[should_panic(expected = "No tests currently use this function")]
    #[test]
    fn we_can_get_unimplemented_error_for_try_consume_post_result_challenge() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        verification_builder
            .try_consume_post_result_challenge()
            .unwrap();
    }

    #[test]
    fn we_can_get_sumcheck_proof_too_small_for_identity() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
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
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
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
                Vec::new(),
                vec![
                    vec![TestScalar::ONE, TestScalar::TWO],
                    vec![-TestScalar::ONE, -TestScalar::TWO],
                ],
                Vec::new(),
                Vec::new(),
            );
        let result = verification_builder
            .try_consume_final_round_mle_evaluations(2)
            .unwrap();
        assert_eq!(result, vec![TestScalar::ONE, TestScalar::TWO]);
    }

    #[test]
    fn we_can_get_error_when_not_enough_evaluations() {
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                vec![vec![TestScalar::ONE]],
                Vec::new(),
                Vec::new(),
            );
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
                Vec::new(),
                Vec::new(),
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
            MockVerificationBuilder::new(
                Vec::new(),
                2,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );
        let error = verification_builder
            .try_consume_bit_distribution()
            .unwrap_err();
        assert!(matches!(error, ProofSizeMismatch::TooFewBitDistributions));
    }
}
