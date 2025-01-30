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
    builder.update_range_length(65536);

    // Create 31 columns, each will collect the corresponding byte from all scalars.
    // 31 because a scalar will only ever have 248 bits set.
    let mut word_columns: Vec<&mut [u16]> = (0..15)
        .map(|_| alloc.alloc_slice_fill_copy(column_data.len(), 0))
        .collect();

    // Decompose scalars to bytes
    let span = span!(Level::DEBUG, "decompose scalars in first round").entered();
    decompose_scalars_to_words(column_data, &mut word_columns);
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
    // Create 31 columns, each will collect the corresponding word from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let span = span!(Level::DEBUG, "allocate word columns in final round builder").entered();
    let mut word_columns: Vec<&mut [u16]> =
        repeat_with(|| alloc.alloc_slice_fill_copy(column_data.len(), 0))
            .take(15)
            .collect();
    span.exit();

    // Allocate space for the eventual inverted word columns by copying word_columns and converting to the required type.
    let span = span!(
        Level::DEBUG,
        "allocate inverted word columns in final round builder"
    )
    .entered();
    let mut inverted_word_columns: Vec<&mut [S]> = word_columns
        .iter_mut()
        .map(|column| alloc.alloc_slice_fill_with(column.len(), |_| S::ZERO))
        .collect();
    span.exit();

    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 65536 elements padded with zeros to match the length of the word columns
    // The size is the larger of 65536 or the number of scalars.
    let word_counts: &mut [i64] = alloc.alloc_slice_fill_with(65536, |_| 0);

    let span = span!(Level::DEBUG, "decompose scalars in final round").entered();
    decompose_scalars_to_words(column_data, &mut word_columns);
    span.exit();

    let word_columns_immut: Vec<&[u16]> = word_columns
        .into_iter()
        .map(|column| &column[..]) // convert &mut [u16] -> &[u16]
        .collect();

    let span = span!(Level::DEBUG, "count_word_occurrences in final round").entered();
    count_word_occurrences(&word_columns_immut, column_data.len(), word_counts);
    span.exit();

    // Retrieve verifier challenge here, *after* Phase 1
    let alpha = builder.consume_post_result_challenge();

    let span = span!(Level::DEBUG, "creating word value lookup table").entered();
    // avoids usize to u16 cast
    let mut word_value_table = vec![0u16; 65536];
    let mut inv_word_values_plus_alpha_table = [S::ZERO; 65536];

    // Same initialization loop
    for i in 0u16..=65535 {
        word_value_table[i as usize] = i;
        inv_word_values_plus_alpha_table[i as usize] = S::from(i);
    }

    let inv_word_vals_plus_alpha_table: &mut [S] = alloc
        .alloc_slice_fill_with(inv_word_values_plus_alpha_table.len(), |i| {
            inv_word_values_plus_alpha_table[i]
        });
    // Add alpha, batch invert, etc.
    slice_ops::add_const::<S, S>(inv_word_vals_plus_alpha_table, alpha);
    slice_ops::batch_inversion(inv_word_vals_plus_alpha_table);
    span.exit();

    let span = span!(Level::DEBUG, "get_logarithmic_derivative in final round").entered();
    get_logarithmic_derivative(
        builder,
        alloc,
        &word_columns_immut,
        alpha,
        &mut inverted_word_columns,
        inv_word_vals_plus_alpha_table,
    );
    span.exit();

    // Produce an MLE over the word values
    let span = span!(
        Level::DEBUG,
        "produce mles and sumcheck polynomials for word val proof"
    )
    .entered();
    prove_word_values(
        alloc,
        alpha,
        builder,
        alloc.alloc_slice_copy(&word_value_table), // give this an explicit lifetime for MLE commitment
        inv_word_vals_plus_alpha_table,
    );
    span.exit();

    // Argue that the sum of all words in each row, minus the count of each
    // word multiplied by the inverted word value, is zero.

    prove_row_zero_sum(
        builder,
        word_counts,
        alloc,
        column_data,
        &inverted_word_columns,
        inv_word_vals_plus_alpha_table,
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
    word_columns: &mut [&mut [u16]],
) where
    T: Copy + Into<S>,
{
    for (i, scalar) in column_data.iter().enumerate() {
        let scalar_array: [u64; 4] = (*scalar).into().into();
        // Convert the [u64; 4] into a slice of bytes
        let scalar_bytes = &cast_slice::<u64, u16>(&scalar_array)[..15];

        // Zip the "columns" and the scalar bytes so we can write them directly
        for (column, &byte) in word_columns[..15].iter_mut().zip(scalar_bytes) {
            column[i] = byte;
        }
    }
}

// Count the individual word occurrences in the decomposed columns.
fn count_word_occurrences(word_columns: &[&[u16]], scalar_count: usize, word_counts: &mut [i64]) {
    for column in word_columns.iter().take(15) {
        for &byte in column.iter().take(scalar_count) {
            word_counts[byte as usize] += 1;
        }
    }
}

/// For a word w and a verifier challenge α, compute
/// wᵢⱼ , and produce an Int. MLE over this column:
///
/// ```text
/// R | Column 0     | Column 1     | Column 2     | ... | Column 31    |
///   |--------------|--------------|--------------|-----|--------------|
/// 1 | w₀,₀         | w₀,₁         | w₀,₂         | ... | w₀,₃₁        |
/// 2 | w₁,₀         | w₁,₁         | w₁,₂         | ... | w₁,₃₁        |
/// 3 | w₂,₀         | w₂,₁         | w₂,₂         | ... | w₂,₃₁        |
///   -------------------------------------------------------------------
///       |               |              |                   |            
///       v               v              v                   v          
///    Int. MLE        Int. MLE       Int. MLE            Int. MLE     
/// ```
///
/// Then, invert each column, producing the modular multiplicative
/// inverse of (wᵢⱼ + α), which is the logarithmic derivative
/// of wᵢⱼ + α:
///
/// ```text
/// R | Column 0     | Column 1     | Column 2     | ... | Column 31     |
///   |--------------|--------------|--------------|-----|---------------|
/// 1 | (w₀,₀ + α)⁻¹ | (w₀,₁ + α)⁻¹ | (w₀,₂ + α)⁻¹ | ... | (w₀,₃₁ + α)⁻¹ |
/// 2 | (w₁,₀ + α)⁻¹ | (w₁,₁ + α)⁻¹ | (w₁,₂ + α)⁻¹ | ... | (w₁,₃₁ + α)⁻¹ |
/// 3 | (w₂,₀ + α)⁻¹ | (w₂,₁ + α)⁻¹ | (w₂,₂ + α)⁻¹ | ... | (w₂,₃₁ + α)⁻¹ |
///   --------------------------------------------------------------------
///       |              |              |                    |            
///       v              v              v                    v          
///    Int. MLE      Int. MLE      Int. MLE             Int. MLE     
/// ```
#[tracing::instrument(
    name = "get_logarithmic_derivative in final round",
    level = "debug",
    skip_all
)]
fn get_logarithmic_derivative<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    word_columns: &[&'a [u16]],
    alpha: S,
    inverted_word_columns: &mut [&mut [S]],
    inv_word_vals_plus_alpha_table: &[S],
) {
    let num_columns = word_columns.len();
    let span = span!(Level::DEBUG, "get_logarithmic_derivative total loop time").entered();

    for col_index in 0..num_columns {
        let byte_column = word_columns[col_index];
        let inv_column = &mut inverted_word_columns[col_index];
        let column_length = byte_column.len();

        let words_inv = alloc.alloc_slice_fill_with(column_length, |row_index| {
            inv_word_vals_plus_alpha_table[byte_column[row_index] as usize]
        });

        builder.produce_intermediate_mle(words_inv as &[_]);

        inv_column.copy_from_slice(words_inv);

        let input_ones =
            alloc.alloc_slice_fill_copy(inverted_word_columns[0].len(), true) as &[_];

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (alpha, vec![Box::new(words_inv as &[_])]),
                (
                    S::one(),
                    vec![Box::new(byte_column as &[_]), Box::new(words_inv as &[_])],
                ),
                (-S::one(), vec![Box::new(input_ones as &[_])]),
            ],
        );
    }
    span.exit();
}

/// Produce the range of possible values that a word can take on,
/// based on the word's bit size, along with an intermediate MLE:
///
/// ```text
/// | Column 0           |
/// |--------------------|
/// |  0                 |
/// |  1                 |
/// |  ...               |
/// |  2ⁿ - 1            |
/// ----------------------
///       |       
///       v  
///    Int. MLE
/// ```
/// Here, `n` represents the bit size of the word (e.g., for an 8-bit word, `2⁸ - 1 = 255`).
///
/// Then, add the verifier challenge α, invert, and produce an
/// intermediate MLE:
///
/// ```text
/// | Column 0
/// |--------------------|
/// | (0 + α)⁻¹          |
/// | (1 + α)⁻¹          |
/// | ...                |
/// | (2ⁿ - 1 + α)⁻¹     |
/// ----------------------
///       |      
///       v        
///    Int. MLE  
/// ```
/// Finally, argue that (`word_values` + α)⁻¹ * (`word_values` + α) - 1 = 0
///
fn prove_word_values<'a, S: Scalar + 'a>(
    alloc: &'a Bump,
    alpha: S,
    builder: &mut FinalRoundBuilder<'a, S>,
    word_val_table: &'a [u16],
    inv_word_vals_plus_alpha_table: &'a [S],
) {
    builder.produce_intermediate_mle(inv_word_vals_plus_alpha_table as &[_]);

    let input_ones = alloc.alloc_slice_fill_copy(65536, true);

    // Argument:
    // (word_values + α)⁻¹ * (word_values + α) - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                alpha,
                vec![Box::new(inv_word_vals_plus_alpha_table as &[_])],
            ),
            (
                S::one(),
                vec![
                    Box::new(inv_word_vals_plus_alpha_table as &[_]),
                    Box::new(word_val_table as &[_]),
                ],
            ),
            (-S::one(), vec![Box::new(input_ones as &[_])]),
        ],
    );
}

/// Argue that the sum of all words in each row, minus the count of each word
/// multiplied by the inverted word value, is zero.
///
/// ```text
/// ∑ (I₀ + I₁ + I₂ ... Iₙ - (C * IN)) = 0
/// ```
///
/// Where:
/// - `I₀ + I₁ + I₂ ... Iₙ` are the inverted word columns.
/// - `C` is the count of each word.
/// - `IN` is the inverted word values column.
fn prove_row_zero_sum<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    word_counts: &'a mut [i64],
    alloc: &'a Bump,
    column_data: &[impl Into<S>],
    inverted_word_columns: &[&mut [S]],
    word_vals_plus_alpha_inv: &'a [S],
) {
    let span = span!(Level::DEBUG, "MLE over word counts in prove row zero sum").entered();
    // Produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(word_counts as &[_]);
    span.exit();

    let span = span!(
        Level::DEBUG,
        "compute sum over columns at row index in prove row zero sum"
    )
    .entered();
    // Compute sum over all columns at each row index (single-threaded)
    let row_sums = alloc.alloc_slice_fill_copy(column_data.len(), S::ZERO);
    for column in inverted_word_columns {
        for (i, &inv_word) in column.iter().enumerate() {
            row_sums[i] += inv_word;
        }
    }
    span.exit();

    let span = span!(
        Level::DEBUG,
        "compute sum over columns at row index in prove row zero sum"
    )
    .entered();
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(row_sums as &[_])]),
            (
                -S::one(),
                vec![
                    Box::new(word_counts as &[_]),
                    Box::new(word_vals_plus_alpha_inv as &[_]),
                ],
            ),
        ],
    );
    span.exit();
}

/// Verify that the prover claim is correct.
///
/// # Panics
///
/// if a column contains values outside of the selected range.
pub(crate) fn verifier_evaluate_range_check<S: Scalar>(
    builder: &mut VerificationBuilder<'_, S>,
    input_column_eval: S,
    input_ones_eval: S,
) -> Result<(), ProofSizeMismatch> {
    // Retrieve the post-result challenge α
    let alpha = builder.try_consume_post_result_challenge()?;
    let chi_ones_65536_eval = builder.try_consume_one_evaluation()?;

    // We will accumulate ∑(wᵢ * 65536ⁱ) in `sum`.
    // Additionally, we'll collect all (wᵢ + α)⁻¹ evaluations in `w_plus_alpha_inv_evals`
    // to use later for the ZeroSum argument.
    let mut sum = S::ZERO;
    let mut w_plus_alpha_inv_evals = Vec::with_capacity(15);

    // Process 31 columns (one per byte in a 248-bit decomposition).
    // Each iteration handles:
    //  - Consuming MLE evaluations for wᵢ and (wᵢ + α)⁻¹
    //  - Verifying that (wᵢ + α)⁻¹ * (wᵢ + α) - 1 = 0
    //  - Accumulating wᵢ * 65536ⁱ into `sum`
    for i in 0..15 {
        // Consume the next MLE evaluations: one for wᵢ, one for (wᵢ + α)⁻¹
        let w_eval = builder.try_consume_first_round_mle_evaluation()?;
        let words_inv = builder.try_consume_final_round_mle_evaluation()?;

        // Compute word_eval = (wᵢ + α) * (wᵢ + α)⁻¹
        // This is used in the subpolynomial check below.
        let word_eval = words_inv * (w_eval + alpha);

        // Compute 65536ⁱ via a small loop (instead of a fold or pow)
        let mut power = S::from(1);
        for _ in 0..i {
            power *= S::from(65536);
        }

        // Argue that ( (wᵢ + α)⁻¹ * (wᵢ + α) ) - 1 = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            word_eval - input_ones_eval,
            2,
        )?;

        // Add wᵢ * 65536ⁱ to our running sum to ensure the entire column is in range
        sum += w_eval * power;

        // Collect the inverse factor for the final ZeroSum argument
        w_plus_alpha_inv_evals.push(words_inv);
    }

    // Ensure the sum of the scalars (interpreted in base 65536) matches
    // the claimed input_column_eval. If not, the column is out of range.
    assert_eq!(
        sum, input_column_eval,
        "Range check failed, column contains values outside of the selected range"
    );

    // Retrieve word_vals_eval (evaluation for w-values)
    // from the builder’s MLE evaluations
    let word_vals_eval = builder
        .mle_evaluations
        .rho_256_evaluation
        .ok_or(ProofSizeMismatch::TooFewSumcheckVariables)?;

    // Retrieve the final-round MLE evaluation for (word_vals + α)⁻¹
    let word_vals_plus_alpha_inv = builder.try_consume_final_round_mle_evaluation()?;

    // Argue that (word_vals + α)⁻¹ * (word_vals + α) - 1 = 0
    let word_value_constraint = word_vals_plus_alpha_inv * (word_vals_eval + alpha);
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        word_value_constraint - chi_ones_65536_eval,
        2,
    )?;

    // The final-round MLE evaluation for word count
    let count_eval = builder.try_consume_final_round_mle_evaluation()?;

    // Sum over all (wᵢ + α)⁻¹ evaluations to get row_sum_eval
    let mut row_sum_eval = S::ZERO;
    for inv_eval in &w_plus_alpha_inv_evals {
        row_sum_eval += *inv_eval;
    }

    // Compute count_eval * (word_vals + α)⁻¹
    let count_value_product_eval = count_eval * word_vals_plus_alpha_inv;

    // Argue that row_sum_eval - (count_eval * (word_vals + α)⁻¹) = 0
    // This ensures consistency of counts vs. actual row sums.
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        row_sum_eval - count_value_product_eval,
        2,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::scalar::{Curve25519Scalar as S, Scalar},
        sql::proof::FinalRoundBuilder,
    };
    use alloc::collections::VecDeque;
    use num_traits::Inv;

    #[test]
    fn we_can_decompose_small_scalars_to_words() {
        let scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();

        let mut word_columns = vec![vec![0; scalars.len()]; 31];
        let mut word_slices: Vec<&mut [u16]> = word_columns
            .iter_mut()
            .map(|column| &mut column[..])
            .collect();

        let mut byte_counts = vec![0; 256];

        // Call the decomposer first
        decompose_scalars_to_words::<S, S>(&scalars, &mut word_slices);

        let word_columns_immut: Vec<&[u16]> = word_slices
            .iter()
            .map(|column| &column[..]) // convert &mut [u16] -> &[u16]
            .collect();

        // Then do the counting
        count_word_occurrences(&word_columns_immut, scalars.len(), &mut byte_counts);

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
        let scalars: Vec<S> = [S::MAX_SIGNED, S::from(u64::MAX), S::from(-1)]
            .iter()
            .map(S::from)
            .collect();

        let mut word_columns = vec![vec![0; scalars.len()]; 31];
        let mut word_slices: Vec<&mut [u16]> = word_columns
            .iter_mut()
            .map(|column| &mut column[..])
            .collect();

        let mut byte_counts = vec![0; 256];

        decompose_scalars_to_words::<S, S>(&scalars, &mut word_slices);

        let word_columns_immut: Vec<&[u16]> =
            word_slices.iter().map(|column| &column[..]).collect();

        count_word_occurrences(&word_columns_immut, scalars.len(), &mut byte_counts);

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
        let mut word_columns: Vec<Vec<u16>> = vec![vec![0; scalars.len()]; 31];

        // Manually set the decomposed words column
        word_columns[0] = [1, 2, 3, 255, 0, 1].to_vec();
        word_columns[1] = [0, 0, 0, 0, 1, 1].to_vec();

        let alpha = S::from(5);

        // Initialize the inverted_word_columns_plus_alpha vector
        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(Vec::as_mut_slice)
            .collect();

        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let mut table = [0u16; 256];
        let mut table_plus_alpha = [S::ZERO; 256];

        for i in 0u16..=255 {
            table[i as usize] = i;
            table_plus_alpha[i as usize] = S::from(&i);
        }

        slice_ops::add_const::<S, S>(&mut table_plus_alpha, alpha);
        slice_ops::batch_inversion(&mut table_plus_alpha);

        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &word_columns.iter().map(|col| &col[..]).collect::<Vec<_>>(),
            alpha,
            &mut word_columns_from_log_deriv,
            &table_plus_alpha,
        );

        let expected_data: [[u16; 6]; 31] = [
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

        let mut word_columns: Vec<Vec<u16>> = vec![vec![0; scalars.len()]; 31];

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

        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];
        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(Vec::as_mut_slice)
            .collect();

        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let mut table = [0u16; 256];
        let mut table_plus_alpha = [S::ZERO; 256];

        for i in 0u16..=255 {
            table[i as usize] = i;
            table_plus_alpha[i as usize] = S::from(&i);
        }
        slice_ops::add_const::<S, S>(&mut table_plus_alpha, alpha);
        slice_ops::batch_inversion(&mut table_plus_alpha);

        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &word_columns.iter().map(|col| &col[..]).collect::<Vec<_>>(),
            alpha,
            &mut word_columns_from_log_deriv,
            &table_plus_alpha,
        );

        let expected_data: [[u16; 2]; 31] = [
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
}
