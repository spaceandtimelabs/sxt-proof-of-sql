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
    base::{polynomial::MultilinearExtension, proof::ProofSizeMismatch, scalar::Scalar, slice_ops},
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use bytemuck::cast_slice;
use core::{cmp::max, iter::repeat};

/// Update the max range length for the range check.
#[allow(dead_code)]
pub(crate) fn first_round_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut FirstRoundBuilder<'a, S>,
    scalars: &[S],
    alloc: &'a Bump,
) {
    builder.update_range_length(256);

    // Create 31 columns, each will collect the corresponding word from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut word_columns: Vec<&mut [u8]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_copy(scalars.len(), 0))
        .collect();
    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // The size is the larger of 256 or the number of scalars.
    let word_counts: &mut [i64] = alloc.alloc_slice_fill_copy(max(256, scalars.len()), 0);

    decompose_scalar_to_words(scalars, &mut word_columns, word_counts);

    for byte_column in &mut word_columns {
        // Allocate words
        let words = alloc.alloc_slice_fill_with(byte_column.len(), |j| S::from(&byte_column[j]));

        // Produce an MLE over words
        builder.produce_intermediate_mle(words as &[_]);
    }
}

/// Prove that a word-wise decomposition of a collection of scalars
/// are all within the range 0 to 2^248.
#[allow(dead_code)]
pub(crate) fn final_round_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    scalars: &[S],
    table_length: usize,
    alloc: &'a Bump,
) {
    // Create 31 columns, each will collect the corresponding word from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut word_columns: Vec<&mut [u8]> = repeat(())
        .take(31)
        .map(|()| alloc.alloc_slice_fill_copy(scalars.len(), 0))
        .collect();

    // Allocate space for the eventual inverted word columns by copying word_columns and converting to the required type.
    let mut inverted_word_columns: Vec<&mut [S]> = word_columns
        .iter_mut()
        .map(|column| alloc.alloc_slice_fill_with(column.len(), |_| S::ZERO))
        .collect();

    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // The size is the larger of 256 or the number of scalars.
    let word_counts: &mut [i64] = alloc.alloc_slice_fill_with(max(256, scalars.len()), |_| 0);

    decompose_scalar_to_words(scalars, &mut word_columns, word_counts);

    // Retrieve verifier challenge here, *after* Phase 1
    let alpha = builder.consume_post_result_challenge();

    get_logarithmic_derivative(
        builder,
        alloc,
        &mut word_columns,
        alpha,
        table_length,
        &mut inverted_word_columns,
    );

    // Produce an MLE over the word values
    prove_word_values(alloc, scalars, alpha, table_length, builder);

    // Argue that the sum of all words in each row, minus the count of each
    // word multiplied by the inverted word value, is zero.
    prove_row_zero_sum(
        builder,
        word_counts,
        alloc,
        scalars,
        &inverted_word_columns,
        alpha,
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
#[allow(dead_code)]
fn decompose_scalar_to_words<'a, S: Scalar + 'a>(
    scalars: &[S],
    word_columns: &mut [&mut [u8]],
    byte_counts: &mut [i64],
) {
    scalars.iter().enumerate().for_each(|(i, scalar)| {
        let scalar_array: [u64; 4] = (*scalar).into(); // Convert scalar to u64 array
        let scalar_bytes_full = cast_slice::<u64, u8>(&scalar_array); // Cast u64 array to u8 slice
        let scalar_bytes = &scalar_bytes_full[..31];

        scalar_bytes
            .iter()
            .enumerate()
            .for_each(|(byte_index, &byte)| {
                word_columns[byte_index][i] = byte;
                byte_counts[byte as usize] += 1;
            });
    });
}

/// For a word w and a verifier challenge α, compute
/// wᵢⱼ , and produce an Int. MLE over this column:
///
/// ```text
/// | Column 0     | Column 1     | Column 2     | ... | Column 31    |
/// |--------------|--------------|--------------|-----|--------------|
/// | w₀,₀         | w₀,₁         | w₀,₂         | ... | w₀,₃₁        |
/// | w₁,₀         | w₁,₁         | w₁,₂         | ... | w₁,₃₁        |
/// | w₂,₀         | w₂,₁         | w₂,₂         | ... | w₂,₃₁        |
/// -------------------------------------------------------------------
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
/// | Column 0     | Column 1     | Column 2     | ... | Column 31     |
/// |--------------|--------------|--------------|-----|---------------|
/// | (w₀,₀ + α)⁻¹ | (w₀,₁ + α)⁻¹ | (w₀,₂ + α)⁻¹ | ... | (w₀,₃₁ + α)⁻¹ |
/// | (w₁,₀ + α)⁻¹ | (w₁,₁ + α)⁻¹ | (w₁,₂ + α)⁻¹ | ... | (w₁,₃₁ + α)⁻¹ |
/// | (w₂,₀ + α)⁻¹ | (w₂,₁ + α)⁻¹ | (w₂,₂ + α)⁻¹ | ... | (w₂,₃₁ + α)⁻¹ |
/// --------------------------------------------------------------------
///       |              |              |                    |            
///       v              v              v                    v          
///    Int. MLE      Int. MLE      Int. MLE             Int. MLE     
/// ```
#[allow(dead_code)]
fn get_logarithmic_derivative<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    word_columns: &mut [&mut [u8]],
    alpha: S,
    table_length: usize,
    inverted_word_columns: &mut [&mut [S]],
) {
    word_columns
        .iter_mut()
        .zip(inverted_word_columns.iter_mut())
        .for_each(|(byte_column, inv_column)| {
            // Allocate words
            let words =
                alloc.alloc_slice_fill_with(byte_column.len(), |j| S::from(&byte_column[j]));

            // Allocate words_inv
            let words_inv =
                alloc.alloc_slice_fill_with(byte_column.len(), |j| S::from(&byte_column[j]));
            slice_ops::add_const::<S, S>(words_inv, alpha);
            slice_ops::batch_inversion(words_inv);

            // Copy words_inv to the corresponding inverted_word_columns[i]
            builder.produce_intermediate_mle(words_inv as &[_]);

            inv_column.copy_from_slice(words_inv);

            let input_ones = alloc.alloc_slice_fill_copy(table_length, true);

            // α * (w + α)⁻¹ + w * (w + α)⁻¹ - 1 = 0
            builder.produce_sumcheck_subpolynomial(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (alpha, vec![Box::new(words_inv as &[_])]),
                    (
                        S::one(),
                        vec![Box::new(words as &[_]), Box::new(words_inv as &[_])],
                    ),
                    (-S::one(), vec![Box::new(input_ones as &[_])]),
                ],
            );
        });
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
#[allow(
    dead_code,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation
)]
fn prove_word_values<'a, S: Scalar + 'a>(
    alloc: &'a Bump,
    scalars: &[S],
    alpha: S,
    table_length: usize,
    builder: &mut FinalRoundBuilder<'a, S>,
) {
    // Allocate from 0 to 255 and pertrub with verifier challenge
    let word_values: &mut [S] =
        alloc.alloc_slice_fill_with(max(256, scalars.len()), |i| S::from(&(i as u8)));

    // Now produce an intermediate MLE over the inverted word values + verifier challenge alpha
    let word_vals_inv: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S")
    });

    slice_ops::add_const::<S, S>(word_vals_inv, alpha);
    slice_ops::batch_inversion(&mut word_vals_inv[..]);
    builder.produce_intermediate_mle(word_vals_inv as &[_]);

    let input_ones = alloc.alloc_slice_fill_copy(table_length, true);

    // Argument:
    // (word_values + α)⁻¹ * (word_values + α) - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (alpha, vec![Box::new(word_vals_inv as &[_])]),
            (
                S::one(),
                vec![
                    Box::new(word_vals_inv as &[_]),
                    Box::new(word_values as &[_]),
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
#[allow(clippy::missing_panics_doc)]
fn prove_row_zero_sum<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    word_counts: &'a mut [i64],
    alloc: &'a Bump,
    scalars: &[S],
    inverted_word_columns: &[&mut [S]],
    alpha: S,
) {
    // Produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(word_counts as &[_]);

    // Allocate row_sums from the bump allocator, ensuring it lives as long as 'a
    let row_sums = alloc.alloc_slice_fill_copy(scalars.len(), S::ZERO);

    // Sum up the corresponding row values using iterators
    for column in inverted_word_columns {
        column.iter().enumerate().for_each(|(i, &inv_word)| {
            row_sums[i] += inv_word;
        });
    }

    // Allocate and store the row sums in a Box using the bump allocator
    let row_sums_box: Box<_> =
        Box::new(alloc.alloc_slice_copy(row_sums) as &[_]) as Box<dyn MultilinearExtension<S>>;

    let word_vals_plus_alpha_inv: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S")
    });

    slice_ops::add_const::<S, S>(word_vals_plus_alpha_inv, alpha);
    slice_ops::batch_inversion(&mut word_vals_plus_alpha_inv[..]);

    // Now pass the vector to the builder
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![row_sums_box]),
            (
                -S::one(),
                vec![
                    Box::new(word_counts as &[_]),
                    Box::new(word_vals_plus_alpha_inv as &[_]),
                ],
            ),
        ],
    );
}

/// Verify that the prover claim is correct.
///
/// # Panics
///
/// if a column contains values outside of the selected range.
#[allow(dead_code)]
pub(crate) fn verifier_evaluate_range_check<S: Scalar>(
    builder: &mut VerificationBuilder<'_, S>,
    input_ones_eval: S,
    input_column_eval: S,
) -> Result<(), ProofSizeMismatch> {
    let alpha = builder.try_consume_post_result_challenge()?;

    let (sum, w_plus_alpha_inv_evals) = (0..31)
        .map(|i| {
            let w_eval = builder.try_consume_first_round_mle_evaluation()?;
            let words_inv = builder.try_consume_final_round_mle_evaluation()?;

            let word_eval = words_inv * (w_eval + alpha);
            let power = (0..i).fold(S::from(1), |acc, _| acc * S::from(256));

            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                word_eval - input_ones_eval,
                2,
            )?;

            Ok((w_eval, words_inv, power))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(
            (S::ZERO, Vec::with_capacity(31)),
            |(sum_acc, mut invs), (w_eval, words_inv, power)| {
                (sum_acc + w_eval * power, {
                    invs.push(words_inv);
                    invs
                })
            },
        );

    assert_eq!(
        sum, input_column_eval,
        "Range check failed, column contains values outside of the selected range"
    );

    let word_vals_eval = builder
        .mle_evaluations
        .rho_256_evaluation
        .ok_or(ProofSizeMismatch::TooFewSumcheckVariables)?;
    // Ensures that we have enough sumcheck variables
    let word_vals_plus_alpha_inv = builder.try_consume_final_round_mle_evaluation()?;
    let word_value_constraint = word_vals_plus_alpha_inv * (word_vals_eval + alpha);

    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        word_value_constraint - input_ones_eval,
        2,
    )?;

    let count_eval = builder.try_consume_final_round_mle_evaluation()?;
    let row_sum_eval: S = w_plus_alpha_inv_evals.iter().copied().sum();
    let count_value_product_eval = count_eval * word_vals_plus_alpha_inv;

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
    use bumpalo::Bump;
    use num_traits::Inv;

    #[test]
    fn we_can_decompose_small_scalars_to_words() {
        let scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();

        let mut word_columns = vec![vec![0; scalars.len()]; 31];
        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut byte_counts = vec![0; 256];

        decompose_scalar_to_words(&scalars, &mut word_slices, &mut byte_counts);

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
        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut byte_counts = vec![0; 256];

        decompose_scalar_to_words(&scalars, &mut word_slices, &mut byte_counts);

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

        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();

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

        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &mut word_slices,
            alpha,
            256,
            &mut word_columns_from_log_deriv,
        );

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
        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];
        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(Vec::as_mut_slice)
            .collect();

        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &mut word_slices,
            alpha,
            256,
            &mut word_columns_from_log_deriv,
        );

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
}
