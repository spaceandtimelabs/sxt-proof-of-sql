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
//! * **Word-Sized Decomposition**: Each scalar is decomposed into its byte-level representation, forming a matrix where
//!   each row corresponds to the decomposition of a scalar and each column corresponds to the bytes from the same position
//!   across all scalars.
//! * **Intermediate MLE Computation**: Multi-linear extensions are computed for each word column and for the count of how
//!   often each word appears.
//! * **Logarithmic Derivative Calculation**: After decomposing the scalars, the verifier's challenge is added to each word,
//!   and the modular multiplicative inverse of this sum is computed, forming a new matrix of logarithmic derivatives.
//!   This matrix is key to constructing range constraints.
//!
//! ## Optimization Opportunities:
//! * **Batch Inversion**: Inversions of large vectors are computationally expensive
//! * **Parallelization**: Single-threaded execution of these operations is a performance bottleneck
use crate::{
    base::{commitment::Commitment, polynomial::MultilinearExtension, scalar::Scalar, slice_ops},
    sql::proof::{CountBuilder, ProofBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use bumpalo::Bump;
use bytemuck::cast_slice;

/// Prove that a word-wise decomposition of a collection of scalars
/// are all within the range 0 to 2^248.
pub fn prover_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    scalars: &mut [S],
    alloc: &'a Bump,
) {
    // Create 31 columns, each will collect the corresponding word from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut word_columns: Vec<&mut [u8]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0))
        .collect();

    // Allocate space for the eventual inverted word columns.
    let mut inverted_word_columns: Vec<&mut [S]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO))
        .collect();

    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // The size is the larger of 256 or the number of scalars.
    let word_counts: &mut [i64] =
        alloc.alloc_slice_fill_with(std::cmp::max(256, scalars.len()), |_| 0);

    decompose_scalar_to_words(scalars, &mut word_columns, word_counts);
    // dbg!(&byte_counts);
    // Retrieve verifier challenge here, after Phase 1
    let alpha = builder.consume_post_result_challenge();

    get_logarithmic_derivative(
        builder,
        alloc,
        &mut word_columns,
        alpha,
        &mut inverted_word_columns,
    );

    prove_word_values(alloc, scalars, alpha, builder);

    // Produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(word_counts as &[_]);

    // Allocate row_sums from the bump allocator, ensuring it lives as long as 'a
    let row_sums = alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO);

    dbg!(row_sums.len());

    // Iterate over each column and sum up the corresponding row values
    for column in inverted_word_columns.iter() {
        // Iterate over each scalar in the column
        for (i, inv_word) in column.iter().enumerate() {
            row_sums[i] += *inv_word;
        }
    }

    // Pass the row_sums reference with the correct lifetime to the builder
    builder.produce_intermediate_mle(row_sums as &[_]);

    // Allocate and store the row sums in a Box using the bump allocator
    let row_sums_box: Box<_> =
        Box::new(alloc.alloc_slice_copy(row_sums) as &[_]) as Box<dyn MultilinearExtension<S>>;

    let inverted_word_values_plus_alpha: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S") + alpha
    });

    slice_ops::batch_inversion(&mut inverted_word_values_plus_alpha[..]);

    // Now pass the vector to the builder
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![row_sums_box]),
            (
                -S::one(),
                vec![
                    Box::new(word_counts as &[_]),
                    Box::new(inverted_word_values_plus_alpha as &[_]),
                ],
            ),
        ],
    );

    dbg!("prover completed");
}

/// Verify the prover claim
pub fn verifier_evaluate_range_check<'a, C: Commitment + 'a>(
    builder: &mut VerificationBuilder<'a, C>,
) {
    let _alpha = builder.consume_post_result_challenge();
    let mut w_plus_alpha_inv_evals: Vec<_> = Vec::with_capacity(31);
    dbg!("made it here");
    // Step 1:
    // Consume the (wᵢⱼ + α)  and (wᵢⱼ + α)⁻¹ MLEs
    for _ in 0..31 {
        let w_plus_alpha_eval = builder.consume_intermediate_mle();
        let w_plus_alpha_inv_eval = builder.consume_intermediate_mle();

        // Store the evaluations of (wᵢⱼ + α)⁻¹
        w_plus_alpha_inv_evals.push(w_plus_alpha_inv_eval);

        // Verify that:
        // (wᵢⱼ + α)⁻¹ * (wᵢⱼ + α) - 1 = 0
        let word_eval =
            (w_plus_alpha_inv_eval * w_plus_alpha_eval) - builder.mle_evaluations.one_evaluation;
        builder.produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            word_eval,
        );
    }

    // Step 2:
    // Consume the (word_values + α)⁻¹ * (word_values + α) MLEs:
    let word_plus_alpha_evals = builder.consume_intermediate_mle();
    let inverted_word_values_eval = builder.consume_intermediate_mle();

    // Verify that:
    // (word_values + α)⁻¹ * (word_values + α) - 1 = 0
    let word_value_eval = (inverted_word_values_eval * word_plus_alpha_evals)
        - builder.mle_evaluations.one_evaluation;

    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        word_value_eval,
    );

    // Consume the word count mle:
    let count_eval = builder.consume_intermediate_mle();

    let row_sum_eval = builder.consume_intermediate_mle();
    let count_value_product_eval = count_eval * inverted_word_values_eval;
    dbg!(row_sum_eval - count_value_product_eval);

    builder.produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        row_sum_eval - count_value_product_eval,
    );
}

/// Get a count of the intermediate MLEs, post-result challenges, and subpolynomials
pub fn count(builder: &mut CountBuilder<'_>) {
    builder.count_intermediate_mles(66);
    builder.count_post_result_challenges(1);
    builder.count_degree(3);
    builder.count_subpolynomials(34);
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
/// Finally, argue that (word_values + α)⁻¹ * (word_values + α) - 1 = 0
fn prove_word_values<'a, S: Scalar + 'a>(
    alloc: &'a Bump,
    scalars: &mut [S],
    alpha: S,
    builder: &mut ProofBuilder<'a, S>,
) {
    // Allocate from 0 to 255 and pertrub with verifier challenge
    let word_values_plus_alpha: &mut [S] = alloc
        .alloc_slice_fill_with(std::cmp::max(256, scalars.len()), |i| {
            S::from(&(i as u8)) + alpha
        });
    builder.produce_intermediate_mle(word_values_plus_alpha as &[_]);

    // Now produce an intermediate MLE over the inverted word values + verifier challenge alpha
    let inverted_word_values_plus_alpha: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S") + alpha
    });
    slice_ops::batch_inversion(&mut inverted_word_values_plus_alpha[..]);
    builder.produce_intermediate_mle(inverted_word_values_plus_alpha as &[_]);

    // Argument:
    // (word_values + α)⁻¹ * (word_values + α) - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![
                    Box::new(word_values_plus_alpha as &[_]),
                    Box::new(inverted_word_values_plus_alpha as &[_]),
                ],
            ),
            (-S::one(), vec![]),
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

/// For a word w and a verifier challenge α, compute
/// wᵢⱼ + α, and produce an Int. MLE over this column:
///
/// ```text
/// | Column 0     | Column 1     | Column 2     | ... | Column 31    |
/// |--------------|--------------|--------------|-----|--------------|
/// | w₀,₀ + α     | w₀,₁ + α     | w₀,₂ + α     | ... | w₀,₃₁ + α    |
/// | w₁,₀ + α     | w₁,₁ + α     | w₁,₂ + α     | ... | w₁,₃₁ + α    |
/// | w₂,₀ + α     | w₂,₁ + α     | w₂,₂ + α     | ... | w₂,₃₁ + α    |
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
fn get_logarithmic_derivative<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    word_columns: &mut [&mut [u8]],
    alpha: S,
    inverted_word_columns: &mut [&mut [S]],
) {
    // Iterate over each column
    for (i, byte_column) in word_columns.iter_mut().enumerate() {
        // Allocate words_plus_alpha
        let words_plus_alpha: &mut [S] =
            alloc.alloc_slice_fill_with(byte_column.len(), |j| S::from(&byte_column[j]) + alpha);

        // Produce an MLE over words_plus_alpha
        builder.produce_intermediate_mle(words_plus_alpha as &[_]);

        // Allocate words_plus_alpha
        let words_plus_alpha_inv: &mut [S] =
            alloc.alloc_slice_fill_with(byte_column.len(), |j| S::from(&byte_column[j]) + alpha);
        slice_ops::batch_inversion(&mut words_plus_alpha_inv[..]);

        builder.produce_intermediate_mle(words_plus_alpha_inv as &[_]);

        // Copy words_plus_alpha to the corresponding inverted_word_columns[i]
        inverted_word_columns[i].copy_from_slice(words_plus_alpha_inv);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![
                        Box::new(words_plus_alpha as &[_]),
                        Box::new(words_plus_alpha_inv as &[_]),
                    ],
                ),
                (-S::one(), vec![]),
            ],
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::scalar::{Curve25519Scalar as S, Scalar},
        sql::{
            proof::ProofBuilder,
            proof_exprs::range_check::{decompose_scalar_to_words, get_logarithmic_derivative},
        },
    };
    use bumpalo::Bump;
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

        let mut word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();

        let alpha = S::from(5);

        // Initialize the inverted_word_columns_plus_alpha vector
        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut word_columns_from_log_deriv: Vec<&mut [S]> = inverted_word_columns_plus_alpha
            .iter_mut()
            .map(|col| col.as_mut_slice())
            .collect();

        let alloc = Bump::new();
        let mut builder = ProofBuilder::new(2, 1, Vec::new());

        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &mut word_slices,
            alpha,
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
            .map(|col| col.as_mut_slice())
            .collect();

        let alloc = Bump::new();
        let mut builder = ProofBuilder::new(2, 1, Vec::new());
        get_logarithmic_derivative(
            &mut builder,
            &alloc,
            &mut word_slices,
            alpha,
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
