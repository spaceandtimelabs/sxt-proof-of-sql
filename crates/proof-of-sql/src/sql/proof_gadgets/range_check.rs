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
    base::{proof::ProofSizeMismatch, scalar::Scalar, slice_ops},
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use bytemuck::cast_slice;
use core::iter::repeat_with;
use tracing::{span, Level};

#[tracing::instrument(name = "range check first round evaluate", level = "debug", skip_all)]
pub(crate) fn first_round_evaluate_range_check<'a, S>(
    builder: &mut FirstRoundBuilder<'a, S>,
    column_data: &[impl Copy + Into<S>],
    alloc: &'a Bump,
) where
    S: Scalar + 'a,
{
    builder.update_range_length(256);
    builder.produce_chi_evaluation_length(256);

    // Decompose scalars to bytes
    let span = span!(Level::DEBUG, "decompose scalars in first round").entered();
    let word_columns = decompose_scalars_to_words(column_data, alloc);
    span.exit();

    // For each column, allocate `words` using the lookup table
    let span = span!(Level::DEBUG, "compute intermediate MLE over word column").entered();
    for byte_column in word_columns {
        // Finally, commit an MLE over these word values
        builder.produce_intermediate_mle(byte_column as &[_]);
    }
    span.exit();
}

/// Prove that a word-wise decomposition of a collection of scalars
/// are all within the range 0 to 2^248.
#[tracing::instrument(name = "range check final round evaluate", level = "debug", skip_all)]
pub(crate) fn final_round_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    column_data: &[impl Copy + Into<S>],
    alloc: &'a Bump,
) {
    let num_rows = column_data.len();
    let span = span!(Level::DEBUG, "decompose scalars in final round").entered();
    // Get the bytewise decomposition of the column of data.
    let word_columns = decompose_scalars_to_words(column_data, alloc);
    span.exit();

    let span = span!(Level::DEBUG, "count_word_occurrences in final round").entered();
    // Initialize a vector to count occurrences of each byte (0-255).
    let word_counts = count_word_occurrences(&word_columns, alloc);
    span.exit();

    // Retrieve verifier challenge here, *after* Phase 1
    let alpha = builder.consume_post_result_challenge();

    // get rho for the first 256 rows
    let rho_256 = alloc.alloc_slice_fill_iter(0u8..=255);

    // get (rho_256 + α)⁻¹
    let rho_256_logarithmic_derivative: &mut [S] =
        alloc.alloc_slice_fill_iter((0..256).map(S::from));
    slice_ops::add_const::<S, S>(rho_256_logarithmic_derivative, alpha);
    slice_ops::batch_inversion(rho_256_logarithmic_derivative);

    let span = span!(Level::DEBUG, "get_logarithmic_derivative total loop time").entered();

    let chi_n = alloc.alloc_slice_fill_copy(num_rows, true) as &[_];

    let row_sums = alloc.alloc_slice_fill_copy(num_rows, S::ZERO);

    for byte_column in word_columns {
        // (wᵢ + α)⁻¹
        let words_inv = get_logarithmic_derivative_from_rho_256_logarithmic_derivative(
            alloc,
            byte_column,
            rho_256_logarithmic_derivative,
        );
        builder.produce_intermediate_mle(words_inv);
        // (wᵢ + α)⁻¹ * (wᵢ + α) - chi_n = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (alpha, vec![Box::new(words_inv)]),
                (
                    S::one(),
                    vec![Box::new(byte_column as &[_]), Box::new(words_inv)],
                ),
                (-S::one(), vec![Box::new(chi_n)]),
            ],
        );
        // Compute sum over all logarithmic derivative columns at each row index (single-threaded)
        for (i, &inv_word) in words_inv.iter().enumerate() {
            row_sums[i] += inv_word;
        }
    }
    span.exit();

    builder.produce_intermediate_mle(rho_256_logarithmic_derivative as &[_]);

    let chi_256 = alloc.alloc_slice_fill_copy(256, true);

    // Argument:
    // (rho_256 + α)⁻¹ * (rho_256 + α) - chi_256 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                alpha,
                vec![Box::new(rho_256_logarithmic_derivative as &[_])],
            ),
            (
                S::one(),
                vec![
                    Box::new(rho_256_logarithmic_derivative as &[_]),
                    Box::new(rho_256 as &[_]),
                ],
            ),
            (-S::one(), vec![Box::new(chi_256 as &[_])]),
        ],
    );

    builder.produce_intermediate_mle(word_counts as &[_]);

    // ∑ⱼ ((∑ᵢ (wᵢⱼ + α)⁻¹) - countⱼ * (ρ₂₅₆,ⱼ + α)⁻¹) = 0 where 0 <= i < 32, and 0 <= j < max(n, 256)
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(row_sums as &[_])]),
            (
                -S::one(),
                vec![
                    Box::new(word_counts as &[_]),
                    Box::new(rho_256_logarithmic_derivative as &[_]),
                ],
            ),
        ],
    );
}

/// Decomposes a scalar to requisite words, additionally tracks the total
/// number of occurrences of each word for later use in the argument.
///
/// ```text
/// | Column 0   | Column 1   | Column 2   | ... | Column 31   |
/// |------------|------------|------------|-----|-------------|
/// |  w₀,₀      |  w₀,₁      |  w₀,₂      | ... |  w₀,₃₁      |
/// |  w₁,₀      |  w₁,₁      |  w₁,₂      | ... |  w₁,₃₁      |
/// |  w₂,₀      |  w₂,₁      |  w₂,₂      | ... |  w₂,₃₁      |
/// ------------------------------------------------------------
/// ```
#[tracing::instrument(
    name = "range check decompose_scalars_to_words",
    level = "debug",
    skip_all
)]
fn decompose_scalars_to_words<'a, T, S: Scalar + 'a>(
    column_data: &[T],
    alloc: &'a Bump,
) -> Vec<&'a [u8]>
where
    T: Copy + Into<S>,
{
    let mut word_columns: Vec<&mut [u8]> =
        repeat_with(|| alloc.alloc_slice_fill_copy(column_data.len(), 0))
            .take(31)
            .collect();
    for (i, scalar) in column_data.iter().enumerate() {
        let scalar_array: [u64; 4] = Into::<S>::into(*scalar).to_limbs();
        // Convert the [u64; 4] into a slice of bytes
        let scalar_bytes = &cast_slice::<u64, u8>(&scalar_array)[..31];

        // Zip the "columns" and the scalar bytes so we can write them directly
        for (column, &byte) in word_columns.iter_mut().zip(scalar_bytes) {
            column[i] = byte;
        }
    }
    word_columns
        .into_iter()
        .map(|column| &column[..]) // convert &mut [u8] -> &[u8]
        .collect()
}

// Count the individual word occurrences in the decomposed columns.
fn count_word_occurrences<'a>(word_columns: &[&[u8]], alloc: &'a Bump) -> &'a mut [i64] {
    let word_counts = alloc.alloc_slice_fill_copy(256, 0);
    for column in word_columns {
        for &byte in *column {
            word_counts[byte as usize] += 1;
        }
    }
    word_counts
}

fn get_logarithmic_derivative_from_rho_256_logarithmic_derivative<'a, S: Scalar>(
    alloc: &'a Bump,
    word_column: &[u8],
    rho_256_logarithmic_derivative: &[S],
) -> &'a [S] {
    alloc.alloc_slice_fill_with(word_column.len(), |row_index| {
        rho_256_logarithmic_derivative[word_column[row_index] as usize]
    })
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
) -> Result<(), ProofSizeMismatch> {
    // Retrieve the post-result challenge α
    let alpha = builder.try_consume_post_result_challenge()?;
    let chi_256_eval = builder.try_consume_chi_evaluation()?.0;

    // We will accumulate ∑(wᵢ * 256ⁱ) in `sum`.
    // Additionally, we'll collect all (wᵢ + α)⁻¹ evaluations in `w_plus_alpha_inv_evals`
    // to use later for the ZeroSum argument.
    let mut word_eval_weighted_sum = S::ZERO;
    let mut word_logarithmic_derivative_eval_sum = S::ZERO;

    // Process 31 columns (one per byte in a 248-bit decomposition).
    // Each iteration handles:
    //  - Consuming MLE evaluations for wᵢ and (wᵢ + α)⁻¹
    //  - Verifying that (wᵢ + α)⁻¹ * (wᵢ + α) - 1 = 0
    //  - Accumulating wᵢ * 256ⁱ into `sum`
    for i in 0..31 {
        // Consume the next MLE evaluations: one for wᵢ, one for (wᵢ + α)⁻¹
        let word_eval = builder.try_consume_first_round_mle_evaluation()?;
        let word_logarithmic_derivative_eval = builder.try_consume_final_round_mle_evaluation()?;

        // Compute 256ⁱ via a small loop (instead of a fold or pow)
        let mut power = S::from(1);
        for _ in 0..i {
            power *= S::from(256);
        }

        // Argue that ( (wᵢ + α)⁻¹ * (wᵢ + α) ) - 1 = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            word_logarithmic_derivative_eval * (word_eval + alpha) - chi_n_eval,
            2,
        )?;

        // Add wᵢ * 256ⁱ to our running sum to ensure the entire column is in range
        word_eval_weighted_sum += word_eval * power;

        // Sum over all (wᵢ + α)⁻¹ evaluations to get row_sum_eval
        word_logarithmic_derivative_eval_sum += word_logarithmic_derivative_eval;
    }

    // Ensure the sum of the scalars (interpreted in base 256) matches
    // the claimed input_column_eval. If not, the column is out of range.
    assert_eq!(
        word_eval_weighted_sum, input_column_eval,
        "Range check failed, column contains values outside of the selected range"
    );

    // Retrieve eval of (0..256)
    let rho_256_eval = builder
        .rho_256_evaluation()
        .ok_or(ProofSizeMismatch::TooFewSumcheckVariables)?;

    // Retrieve the final-round MLE evaluation for (rho_256 + α)⁻¹
    let rho_256_logarithmic_derivative_eval = builder.try_consume_final_round_mle_evaluation()?;

    // Argue that (rho_256 + α)⁻¹ * (rho_256 + α) - 1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        rho_256_logarithmic_derivative_eval * (rho_256_eval + alpha) - chi_256_eval,
        2,
    )?;

    // The final-round MLE evaluation for word count
    let count_eval = builder.try_consume_final_round_mle_evaluation()?;

    // Compute count_eval * (word_vals + α)⁻¹
    let count_value_product_eval = count_eval * rho_256_logarithmic_derivative_eval;

    // Argue that row_sum_eval - (count_eval * (word_vals + α)⁻¹) = 0
    // This ensures consistency of counts vs. actual row sums.
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        word_logarithmic_derivative_eval_sum - count_value_product_eval,
        2,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar as S,
        sql::proof::mock_verification_builder::run_verify_for_each_row,
    };
    use core::convert::identity;
    use num_traits::Inv;
    use std::collections::VecDeque;

    #[test]
    fn we_can_decompose_small_scalars_to_words() {
        let alloc = Bump::new();
        let scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();

        // Call the decomposer first
        let word_columns = decompose_scalars_to_words::<S, S>(&scalars, &alloc);

        // Then do the counting
        let byte_counts = count_word_occurrences(&word_columns, &alloc);

        let mut expected_word_columns = vec![vec![0; scalars.len()]; 31];
        expected_word_columns[0] = vec![1, 2, 3, 255, 0, 1];
        expected_word_columns[1] = vec![0, 0, 0, 0, 1, 1];
        // expected_word_columns[2..] is filled with 0s.
        let mut expected_byte_counts = vec![0; 256];
        expected_byte_counts[0] = 31 * 6 - 7;
        expected_byte_counts[1] = 4;
        expected_byte_counts[2] = 1;
        expected_byte_counts[3] = 1;
        // expected_byte_counts[4..255] is filled with 0s.
        expected_byte_counts[255] = 1;

        assert_eq!(word_columns, expected_word_columns);
        assert_eq!(byte_counts, expected_byte_counts);
    }

    #[test]
    fn we_can_decompose_large_scalars_to_words() {
        let alloc = Bump::new();
        let scalars: Vec<S> = [S::MAX_SIGNED, S::from(u64::MAX), S::from(-1)]
            .iter()
            .map(S::from)
            .collect();

        let word_columns = decompose_scalars_to_words::<S, S>(&scalars, &alloc);
        let byte_counts = count_word_occurrences(&word_columns, &alloc);

        let expected_word_columns = [
            [246, 255, 236],
            [233, 255, 211],
            [122, 255, 245],
            [46, 255, 92],
            [141, 255, 26],
            [49, 255, 99],
            [9, 255, 18],
            [44, 255, 88],
            [107, 0, 214],
            [206, 0, 156],
            [123, 0, 247],
            [81, 0, 162],
            [239, 0, 222],
            [124, 0, 249],
            [111, 0, 222],
            [10, 0, 20],
            // expected_word_columns[16..] is filled with 0s.
        ];

        let mut expected_byte_counts_hardcoded = vec![0; 256];
        expected_byte_counts_hardcoded[0] = 53;
        expected_byte_counts_hardcoded[9] = 1;
        expected_byte_counts_hardcoded[10] = 1;
        expected_byte_counts_hardcoded[18] = 1;
        expected_byte_counts_hardcoded[20] = 1;
        expected_byte_counts_hardcoded[26] = 1;
        expected_byte_counts_hardcoded[44] = 1;
        expected_byte_counts_hardcoded[46] = 1;
        expected_byte_counts_hardcoded[49] = 1;
        expected_byte_counts_hardcoded[81] = 1;
        expected_byte_counts_hardcoded[88] = 1;
        expected_byte_counts_hardcoded[92] = 1;
        expected_byte_counts_hardcoded[99] = 1;
        expected_byte_counts_hardcoded[107] = 1;
        expected_byte_counts_hardcoded[111] = 1;
        expected_byte_counts_hardcoded[122] = 1;
        expected_byte_counts_hardcoded[123] = 1;
        expected_byte_counts_hardcoded[124] = 1;
        expected_byte_counts_hardcoded[141] = 1;
        expected_byte_counts_hardcoded[156] = 1;
        expected_byte_counts_hardcoded[162] = 1;
        expected_byte_counts_hardcoded[206] = 1;
        expected_byte_counts_hardcoded[211] = 1;
        expected_byte_counts_hardcoded[214] = 1;
        expected_byte_counts_hardcoded[222] = 2;
        expected_byte_counts_hardcoded[233] = 1;
        expected_byte_counts_hardcoded[236] = 1;
        expected_byte_counts_hardcoded[239] = 1;
        expected_byte_counts_hardcoded[245] = 1;
        expected_byte_counts_hardcoded[246] = 1;
        expected_byte_counts_hardcoded[247] = 1;
        expected_byte_counts_hardcoded[249] = 1;
        expected_byte_counts_hardcoded[255] = 8;

        assert_eq!(word_columns[..16], expected_word_columns);
        assert_eq!(byte_counts, expected_byte_counts_hardcoded);
    }

    #[test]
    fn we_can_obtain_logarithmic_derivative_from_small_scalar() {
        let scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();
        let mut word_columns: Vec<Vec<u8>> = vec![vec![0; scalars.len()]; 31];

        // Manually set the decomposed words column
        word_columns[0] = [1, 2, 3, 255, 0, 1].to_vec();
        word_columns[1] = [0, 0, 0, 0, 1, 1].to_vec();

        let alpha = S::from(5);

        let alloc = Bump::new();

        let mut table_plus_alpha = [S::ZERO; 256];

        for i in 0u8..=255 {
            table_plus_alpha[i as usize] = S::from(&i);
        }

        slice_ops::add_const::<S, S>(&mut table_plus_alpha, alpha);
        slice_ops::batch_inversion(&mut table_plus_alpha);

        let word_columns_from_log_deriv: Vec<_> = word_columns
            .iter()
            .map(|word_column| {
                get_logarithmic_derivative_from_rho_256_logarithmic_derivative(
                    &alloc,
                    word_column,
                    &table_plus_alpha,
                )
            })
            .collect();

        let expected_data: [[u8; 6]; 31] = [
            [1, 2, 3, 255, 0, 1],
            [0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
        ];

        // Invert the expected data and add the verifier challenge
        let expected_columns: Vec<Vec<S>> = expected_data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&w| (S::from(w) + alpha).inv().unwrap_or(S::ZERO))
                    .collect()
            })
            .collect();

        // Perform assertion for all columns at once
        assert_eq!(word_columns_from_log_deriv, expected_columns);
    }

    #[test]
    fn we_can_obtain_logarithmic_derivative_from_large_scalar() {
        let scalars: Vec<S> = [u64::MAX, u64::MAX].iter().map(S::from).collect();

        let mut word_columns: Vec<Vec<u8>> = vec![vec![0; scalars.len()]; 31];

        // Manually set the decomposed words column.
        // Its helpful to think of this transposed, i.e.
        // Scalar 1:  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  00  00  00  ...
        // Scalar 2:  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  FF  00  00  00  ...
        word_columns[0] = [0xFF, 0xFF].to_vec();
        word_columns[1] = [0xFF, 0xFF].to_vec();
        word_columns[2] = [0xFF, 0xFF].to_vec();
        word_columns[3] = [0xFF, 0xFF].to_vec();
        word_columns[4] = [0xFF, 0xFF].to_vec();
        word_columns[5] = [0xFF, 0xFF].to_vec();
        word_columns[6] = [0xFF, 0xFF].to_vec();
        word_columns[7] = [0xFF, 0xFF].to_vec();
        word_columns[8] = [0xFF, 0xFF].to_vec();
        word_columns[9] = [0xFF, 0xFF].to_vec();
        word_columns[10] = [0xFF, 0xFF].to_vec();
        word_columns[11] = [0xFF, 0xFF].to_vec();
        word_columns[12] = [0xFF, 0xFF].to_vec();
        word_columns[13] = [0xFF, 0xFF].to_vec();
        word_columns[14] = [0xFF, 0xFF].to_vec();
        word_columns[15] = [0xFF, 0xFF].to_vec();

        // Simulate a verifier challenge, then prepare storage for
        // 1 / (word + alpha)
        let alpha = S::from(5);

        let alloc = Bump::new();

        let mut table = [0u8; 256];
        let mut table_plus_alpha = [S::ZERO; 256];

        for i in 0u8..=255 {
            table[i as usize] = i;
            table_plus_alpha[i as usize] = S::from(&i);
        }
        slice_ops::add_const::<S, S>(&mut table_plus_alpha, alpha);
        slice_ops::batch_inversion(&mut table_plus_alpha);

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let word_columns_from_log_deriv: Vec<_> = word_columns
            .iter()
            .map(|word_column| {
                get_logarithmic_derivative_from_rho_256_logarithmic_derivative(
                    &alloc,
                    word_column,
                    &table_plus_alpha,
                )
            })
            .collect();

        let expected_data: [[u8; 2]; 31] = [
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0xFF, 0xFF],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
        ];

        // Invert the expected data and add the verifier challenge, producing
        // columns containing 1 / (word + alpha)
        let expected_columns: Vec<Vec<S>> = expected_data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&w| (S::from(w) + alpha).inv().unwrap_or(S::ZERO))
                    .collect()
            })
            .collect();

        assert_eq!(word_columns_from_log_deriv, expected_columns);
    }

    #[test]
    fn we_can_verify_simple_range_check() {
        // First round
        let alloc = Bump::new();
        let column_data = &[5i64, 0, 3, 28888, 400];
        let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(5);
        first_round_evaluate_range_check(&mut first_round_builder, column_data, &alloc);
        first_round_builder.request_post_result_challenges(1);

        // Final Round
        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(2, VecDeque::from([TestScalar::TEN]));
        final_round_evaluate_range_check(&mut final_round_builder, column_data, &alloc);

        // Verification
        let mock_verification_builder = run_verify_for_each_row(
            5,
            &first_round_builder,
            &final_round_builder,
            Vec::from([TestScalar::TEN]),
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
}
