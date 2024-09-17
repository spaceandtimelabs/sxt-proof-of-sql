//! Decomposes a column of scalars into a matrix of words, so that each word column can be
//! used to produce an intermediate multi-linear extension. Produces intermediate MLEs for:
//! * each column of words
//! * the count of how many times each word occurs
//!
//! And anchored MLEs for:
//! * all possible byte values
//!
//! ## Word-sized decomposition:
//!
//! Each row represents the byte decomposition of a scalar, and each column contains the bytes from
//! the same byte position across all scalars. First, we produce this word-wise decomposition,
//! as well as computing intermediate MLEs over the word columns:
//!
//! ```text
//! | Column 0           | Column 1           | Column 2           | ... | Column 31           |
//! |--------------------|--------------------|--------------------|-----|---------------------|
//! | Byte 0 of Scalar 0 | Byte 1 of Scalar 0 | Byte 2 of Scalar 0 | ... | Byte 31 of Scalar 0 |
//! | Byte 0 of Scalar 1 | Byte 1 of Scalar 1 | Byte 2 of Scalar 1 | ... | Byte 31 of Scalar 1 |
//! | Byte 0 of Scalar 2 | Byte 1 of Scalar 2 | Byte 2 of Scalar 2 | ... | Byte 31 of Scalar 2 |
//! --------------------------------------------------------------------------------------------
//!          |                   |                    |                          |            
//!          v                   v                    v                          v          
//!   intermediate MLE    intermediate MLE     intermediate MLE           intermediate MLE     
//! ```
//!
//! A column containing every single possible value the word can take is established and
//! populated. An anchored MLE is produced over this column, since the verifier knows the range
//! of the words. A column containing the counts of all of word occurrences in the decomposition
//! matrix is established, and an intermediate MLE over this column is produced.
//!
//! Then, the challenge from the verifier is added to each word, and this sum is inverted. The
//! columns, now containing the logarithmic derivative of (alpha + word), form a new
//! matrix. The MLEs over the columns in this new matrix are computed:
//!
//! ```text
//! | Column 0             | Column 1             | Column 2             | . | Column 31             |
//! |----------------------|----------------------|----------------------|---|-----------------------|
//! | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
//! | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
//! | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
//! --------------------------------------------------------------------------------------------------
//!            |                     |                      |                              |            
//!            v                     v                      v                              v          
//!     intermediate MLE      intermediate MLE       intermediate MLE               intermediate MLE     
//! ```
//!
//! This new matrix of logarithmic derivatives, and the original word decomposition, are
//! sufficient to establish constraints for verification.
//!
//! ## Bottlenecks
//! * batch inversion, we should try to do as few of these as possible
//! * single-threaded evaluation; we can likely apply rayon or similar here

use crate::{
    base::{commitment::Commitment, scalar::Scalar, slice_ops},
    sql::proof::{CountBuilder, ProofBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use bumpalo::Bump;
use bytemuck::cast_slice;

/// Creates word columns and produces intermediate MLEs and constraints over
/// their requisite transformations.
pub fn prover_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    scalars: &mut [S],
    alloc: &'a Bump,
) {
    // Create 31 columns, each will collect the corresponding byte from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut word_columns: Vec<&mut [u8]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0))
        .collect();

    // Allocate space for the eventual inverted word columns
    let mut inverted_word_columns: Vec<&mut [S]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO))
        .collect();

    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // The size is the larger of 256 or the number of scalars.
    let byte_counts: &mut [i64] =
        alloc.alloc_slice_fill_with(std::cmp::max(256, scalars.len()), |_| 0);

    // Store a vec of references to word slices for use following
    // retrieval of the verifier challenge
    let all_scalar_bytes = Vec::with_capacity(scalars.len());

    decompose_scalar_to_words(scalars, &mut word_columns, byte_counts);

    // Retrieve verifier challenge here, after Phase 1
    let alpha = builder.consume_post_result_challenge();

    get_logarithmic_derivative(&all_scalar_bytes, alpha, &mut inverted_word_columns);

    // Produce an MLE over each column of words
    for word_column in word_columns {
        builder.produce_intermediate_mle(word_column as &[_]);
    }

    // Produce an MLE over each (word + alpha)^-1 column
    for inverted_word_column in inverted_word_columns {
        builder.produce_intermediate_mle(inverted_word_column as &[_]);
    }

    // Allocate and initialize byte_values to represent the range of possible word values
    // from 0 to 255.
    let word_values_plus_alpha: &mut [S] = alloc
        .alloc_slice_fill_with(std::cmp::max(256, scalars.len()), |i| {
            S::from(&(i as u8)) + alpha
        });

    // Next produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(byte_counts as &[_]);

    // Now produce an intermediate MLE over the inverted word values + verifier challenge alpha
    let inverted_word_values: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S") + alpha
    });
    slice_ops::batch_inversion(&mut inverted_word_values[..]);
    builder.produce_intermediate_mle(inverted_word_values as &[_]);

    // Phase 3: Prove
    // (word_values + alpha) * (word_values + alpha)^(-1) - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![
                    Box::new(word_values_plus_alpha as &[_]),
                    Box::new(inverted_word_values as &[_]),
                ],
            ),
            (-S::one(), vec![]),
        ],
    );
}

/// Verify the prover claim
pub fn verifier_evaluate_range_check<'a, C: Commitment + 'a>(
    builder: &mut VerificationBuilder<'a, C>,
) {
    builder.consume_post_result_challenge();
    for _ in 0..64 {
        builder.consume_intermediate_mle();
        let one_eval = builder.mle_evaluations.one_evaluation;
        let res_eval = builder.consume_result_mle();
        let eval = builder.mle_evaluations.random_evaluation * (one_eval * res_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
}

/// Get a count of the intermediate MLEs, post-result challenges, and subpolynomials
pub fn count(builder: &mut CountBuilder<'_>) {
    builder.count_intermediate_mles(64);
    builder.count_post_result_challenges(1);
    builder.count_degree(2);
    builder.count_subpolynomials(1);
}

// Decomposes a scalar to requisite words, additionally tracks the total
// number of occurences of each word for later use in the argument.
fn decompose_scalar_to_words<'a, S: Scalar + 'a>(
    scalars: &mut [S],
    word_columns: &mut [&mut [u8]],
    byte_counts: &mut [i64],
) {
    for (i, scalar) in scalars.iter().enumerate() {
        let scalar_array: [u64; 4] = (*scalar).into(); // Convert scalar to u64 array
        let scalar_bytes_full = cast_slice::<u64, u8>(&scalar_array); // Cast u64 array to u8 slice
        let scalar_bytes = &scalar_bytes_full[..31];

        // Populate the columns of the words table with decomposition of scalar:
        for (byte_index, &byte) in scalar_bytes.iter().enumerate() {
            // Each column in word_columns is for a specific byte position across all scalars
            word_columns[byte_index][i] = byte;
            byte_counts[byte as usize] += 1;
        }
    }
}

// For a word w and a verifier challenge alpha, compute
// 1 / (word + alpha), which is the modular multiplicative
// inverse of (word + alpha) in the scalar field.
fn get_logarithmic_derivative<'a, S: Scalar + 'a>(
    byte_columns: &[&mut [u8]],
    alpha: S,
    inverted_word_columns: &mut [&mut [S]],
) {
    // Iterate over each column
    for (i, byte_column) in byte_columns.iter().enumerate() {
        // Convert bytes to field elements and add alpha
        let mut terms_to_invert: Vec<S> = byte_column.iter().map(|w| S::from(w) + alpha).collect();

        // Invert all the terms in the column at once
        slice_ops::batch_inversion(&mut terms_to_invert);

        // Assign the inverted values back to the inverted_word_columns
        inverted_word_columns[i].copy_from_slice(&terms_to_invert);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::scalar::{Curve25519Scalar as S, Scalar},
        sql::proof_exprs::range_check::{decompose_scalar_to_words, get_logarithmic_derivative},
    };
    use num_traits::Inv;

    #[test]
    fn we_can_decompose_small_scalars_to_words() {
        let mut scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();

        let mut word_columns = vec![vec![0; scalars.len()]; 31];
        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut byte_counts = vec![0; 256];

        decompose_scalar_to_words(&mut scalars, &mut word_slices, &mut byte_counts);

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
        let mut scalars: Vec<S> = [S::MAX_SIGNED, S::from(u64::MAX), S::from(-1)]
            .iter()
            .map(S::from)
            .collect();

        let mut word_columns = vec![vec![0; scalars.len()]; 31];
        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut byte_counts = vec![0; 256];

        decompose_scalar_to_words(&mut scalars, &mut word_slices, &mut byte_counts);

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

        let word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();

        let alpha = S::from(5);

        // Initialize the inverted_word_columns_plus_alpha vector
        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(|col| col.as_mut_slice())
            .collect();

        get_logarithmic_derivative(&word_slices, alpha, &mut word_columns_from_log_deriv);

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
        let word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();
        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];
        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(|col| col.as_mut_slice())
            .collect();

        get_logarithmic_derivative(&word_slices, alpha, &mut word_columns_from_log_deriv);

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
