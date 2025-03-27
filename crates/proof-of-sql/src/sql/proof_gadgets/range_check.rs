//! Implements a cryptographic range check using logarithmic derivatives to decompose a column of scalars
//! into a matrix of words. This method leverages the properties of logarithmic derivatives to efficiently
//! verify range proofs in a zero-knowledge setting by performing word-wise decompositions, intermediate MLEs,
//! and modular inversions.
//!
//! The approach builds on the techniques outlined in the paper "Multivariate Lookups Based on Logarithmic
//! Derivatives" [ePrint 2022/1530](https://eprint.iacr.org/2022/1530.pdf), which characterizes the use of
//! logarithmic derivatives to perform multivariate lookups in cryptographic protocols.
//!
//! ## Key Steps:
//! * Word-Sized Decomposition: Each scalar is decomposed into its byte-level representation, forming a matrix where
//!   each row corresponds to the decomposition of a scalar and each column corresponds to the bytes from the same position
//!   across all scalars.
//! * Intermediate MLE Computation: Multi-linear extensions are computed for each word column and for the count of how
//!   often each word appears.
//! * Logarithmic Derivative Calculation: After decomposing the scalars, the verifier's challenge is added to each word,
//!   and the modular multiplicative inverse of this sum is computed, forming a new matrix of logarithmic derivatives.
//!   This matrix is key to constructing range constraints.
//!
//! ## Optimization Opportunities:
//! * Batch Inversion: Inversions of large vectors are computationally expensive
//! * Parallelization: Single-threaded execution of these operations is a performance bottleneck
use crate::{
    base::{
        byte::{
            byte_matrix_utils::{compute_varying_byte_matrix, get_word_counts},
            ByteDistribution,
        },
        proof::{ProofError, ProofSizeMismatch},
        scalar::{Scalar, ScalarExt},
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::get_logarithmic_derivative,
    },
};
use alloc::{boxed::Box, vec};
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shl;

#[tracing::instrument(name = "range check first round evaluate", level = "debug", skip_all)]
pub(crate) fn first_round_evaluate_range_check<'a, S>(
    builder: &mut FirstRoundBuilder<'a, S>,
    column_data: &[impl Copy + Into<S>],
    alloc: &'a Bump,
) where
    S: Scalar + 'a,
{
    // One of the commitments is column of all possible words, of which there are 256.
    builder.update_range_length(256);
    builder.produce_chi_evaluation_length(256);

    // find the byte columns that are constant
    let word_byte_distribution = ByteDistribution::new(column_data);

    // find the byte columns that vary
    let varying_columns = compute_varying_byte_matrix(column_data, &word_byte_distribution);

    // commit to the constant byte columns
    builder.produce_byte_distribution(word_byte_distribution);

    for byte_column in varying_columns {
        // commit to each varying column
        builder
            .produce_intermediate_mle(alloc.alloc_slice_fill_iter(byte_column.into_iter()) as &[_]);
    }
}

#[tracing::instrument(name = "range check final round evaluate", level = "debug", skip_all)]
pub(crate) fn final_round_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    column_data: &[impl Copy + Into<S>],
    alloc: &'a Bump,
) {
    // get chi_256
    let chi_256 = alloc.alloc_slice_fill_copy(256, 1u8);

    // get chi_n
    let chi_n = alloc.alloc_slice_fill_copy(column_data.len(), 1u8);

    // get row
    let rho_256 = alloc.alloc_slice_fill_with(256, |i| u8::try_from(i).unwrap());

    // get alpha
    let alpha = builder.consume_post_result_challenge();

    // get the varying byte columns
    let word_byte_distribution = ByteDistribution::new(column_data);
    let varying_columns = compute_varying_byte_matrix(column_data, &word_byte_distribution);

    // get the inverses of the varying columns plus alpha and commit to them
    let varying_inverse_columns = varying_columns
        .clone()
        .map(|column| get_logarithmic_derivative(alloc, column, alpha));
    for (column, inverse_column) in varying_columns.clone().zip(varying_inverse_columns.clone()) {
        builder.produce_intermediate_mle(inverse_column);
        // (wordᵢ + α) * (wordᵢ + α)⁻¹ - 1 = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (alpha, vec![Box::new(inverse_column)]),
                (
                    S::one(),
                    vec![
                        Box::new(alloc.alloc_slice_fill_iter(column.into_iter()) as &[_]),
                        Box::new(inverse_column as &[_]),
                    ],
                ),
                (-S::one(), vec![Box::new(chi_n as &[_])]),
            ],
        );
    }

    // calculate the inverses of all 256 words plus alpha
    let rho_inverse = get_logarithmic_derivative(alloc, (0..256).collect(), alpha);

    // commit these inverses
    builder.produce_intermediate_mle(rho_inverse as &[_]);

    // (rhoᵢ + α) * (rhoᵢ + α)⁻¹ - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (alpha, vec![Box::new(rho_inverse as &[_])]),
            (
                S::one(),
                vec![Box::new(rho_256 as &[_]), Box::new(rho_inverse as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_256 as &[_])]),
        ],
    );

    // get the counts of all bytes that belong to varying columns in the data
    let word_counts = get_word_counts(alloc, varying_columns);

    // commit the counts
    builder.produce_intermediate_mle(word_counts);

    // get the sum of each row's inverse bytes
    let varying_column_sum = alloc.alloc_slice_fill_copy(column_data.len(), S::ZERO);
    for varying_inverse_column in varying_inverse_columns {
        for (row_sum, byte) in varying_column_sum.iter_mut().zip(varying_inverse_column) {
            *row_sum += *byte;
        }
    }

    // ∑ (rho + α)⁻¹ * count - ∑∑ (word + α)⁻¹ = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (
                S::one(),
                vec![Box::new(rho_inverse as &[_]), Box::new(word_counts as &[_])],
            ),
            (-S::one(), vec![Box::new(varying_column_sum as &[_])]),
        ],
    );
}

/// Verify that the prover claim is correct.
///
/// # Panics
///
/// if a column contains values outside of the selected range.
pub(crate) fn verifier_evaluate_range_check<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    input_column_eval: S,
    chi_n_eval: S,
) -> Result<(), ProofError> {
    // Retrieve the post-result challenge α
    let alpha = builder.try_consume_post_result_challenge()?;
    let chi_ones_256_eval = builder.try_consume_chi_evaluation()?;

    let word_byte_distribution = builder.try_consume_byte_distribution()?;

    let words = builder.try_consume_first_round_mle_evaluations(
        word_byte_distribution.varying_byte_count().into(),
    )?;
    let word_inverses = builder.try_consume_final_round_mle_evaluations(
        word_byte_distribution.varying_byte_count().into(),
    )?;

    for (word, word_inverse) in words.iter().copied().zip(word_inverses.iter().copied()) {
        // Argue that ( (wᵢ + α)⁻¹ * (wᵢ + α) ) - 1 = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            word_inverse * (word + alpha) - chi_n_eval,
            2,
        )?;
    }

    let sum = words
        .into_iter()
        .zip(word_byte_distribution.varying_byte_indices())
        .fold(
            (S::from_wrapping(word_byte_distribution.constant_mask())
                - S::from_wrapping(U256::ONE.shl(255)))
                * chi_n_eval,
            |acc, (word, i)| acc + word * S::from_wrapping(U256::ONE.shl(i)),
        );

    // Retrieve word_vals_eval (evaluation for w-values)
    // from the builder’s MLE evaluations
    let word_vals_eval = builder
        .rho_256_evaluation()
        .ok_or(ProofSizeMismatch::TooFewSumcheckVariables)?;

    // Retrieve the final-round MLE evaluation for (word_vals + α)⁻¹
    let word_vals_plus_alpha_inv = builder.try_consume_final_round_mle_evaluation()?;

    // Argue that (word_vals + α)⁻¹ * (word_vals + α) - 1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        word_vals_plus_alpha_inv * (word_vals_eval + alpha) - chi_ones_256_eval,
        2,
    )?;

    // The final-round MLE evaluation for word count
    let count_eval = builder.try_consume_final_round_mle_evaluation()?;

    // Sum over all (wᵢ + α)⁻¹ evaluations to get row_sum_eval
    let row_sum_eval = word_inverses
        .into_iter()
        .fold(S::ZERO, |acc, inv| acc + inv);

    // Argue that row_sum_eval - (count_eval * (word_vals + α)⁻¹) = 0
    // This ensures consistency of counts vs. actual row sums.
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        row_sum_eval - count_eval * word_vals_plus_alpha_inv,
        2,
    )?;

    // Ensure the sum of the scalars (interpreted in base 256) matches
    // the claimed input_column_eval. If not, the column is out of range.
    (sum == input_column_eval)
        .then_some(())
        .ok_or(ProofError::VerificationError {
            error: "Range check failed, column contains values outside of the selected range",
        })
}

#[cfg(test)]
mod tests {
    use super::{
        final_round_evaluate_range_check, first_round_evaluate_range_check,
        verifier_evaluate_range_check,
    };
    use crate::{
        base::{
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{
            mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
            FirstRoundBuilder,
        },
    };
    use alloc::collections::VecDeque;
    use bumpalo::Bump;
    use core::convert::identity;

    #[test]
    fn we_can_verify_simple_range_check() {
        // First round
        let alloc = Bump::new();
        let column_data = &[5i64, -12, 3, 28888, -400];
        let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(5);
        first_round_evaluate_range_check(&mut first_round_builder, column_data, &alloc);

        // Final Round
        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(2, VecDeque::from([TestScalar::TEN]));
        final_round_evaluate_range_check(&mut final_round_builder, column_data, &alloc);

        // Verification
        let mock_verification_builder = run_verify_for_each_row(
            5,
            &first_round_builder,
            Vec::from([TestScalar::TEN]),
            &final_round_builder,
            3,
            |verification_builder, chi_eval, evaluation_point| {
                verifier_evaluate_range_check(
                    verification_builder,
                    column_data.inner_product(evaluation_point),
                    chi_eval,
                )
                .unwrap();
            },
        );

        assert!(mock_verification_builder
            .get_identity_results()
            .iter()
            .all(|v| v.iter().copied().all(identity)));
        assert!(mock_verification_builder
            .get_zero_sum_results()
            .iter()
            .copied()
            .all(identity));
    }

    // 
    #[test]
    fn we_can_reject_range_check_verification_if_bytes_are_rearranged(){

    }
}
