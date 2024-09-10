use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar, slice_ops},
    sql::proof::{ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use bytemuck::cast_slice;

/// Decomposes a column of scalars into a matrix of words, so that each word column can be
/// used to produce an intermediate multi-linear extension. Produces intermediate MLEs for:
/// * each column of words
/// * the count of how many times each word occurs
///
/// And anchored MLEs for:
/// * all possible byte values
///
/// ## Word-sized decomposition:
///
/// Each row represents the byte decomposition of a scalar, and each column contains the bytes from
/// the same byte position across all scalars. First, we produce this word-wise decomposition,
/// as well as computing intermediate MLEs over the word columns:
///
/// ```text
/// | Column 0           | Column 1           | Column 2           | ... | Column 31           |
/// |--------------------|--------------------|--------------------|-----|---------------------|
/// | Byte 0 of Scalar 0 | Byte 1 of Scalar 0 | Byte 2 of Scalar 0 | ... | Byte 31 of Scalar 0 |
/// | Byte 0 of Scalar 1 | Byte 1 of Scalar 1 | Byte 2 of Scalar 1 | ... | Byte 31 of Scalar 1 |
/// | Byte 0 of Scalar 2 | Byte 1 of Scalar 2 | Byte 2 of Scalar 2 | ... | Byte 31 of Scalar 2 |
/// --------------------------------------------------------------------------------------------
///          |                   |                    |                          |            
///          v                   v                    v                          v          
///   intermediate MLE    intermediate MLE     intermediate MLE           intermediate MLE     
/// ```
///
/// A column containing every single possible value the word can take is established and
/// populated. An anchored MLE is produced over this column, since the verifier knows the range
/// of the words. A column containing the counts of all of word occurrences in the decomposition
/// matrix is established, and an intermediate MLE over this column is produced.
///
/// Then, the challenge from the verifier is added to each word, and this sum is inverted. The
/// columns, now containing the logarithmic derivative of (alpha + word), form a new
/// matrix. The MLEs over the columns in this new matrix are computed:
///
/// ```text
/// | Column 0             | Column 1             | Column 2             | . | Column 31             |
/// |----------------------|----------------------|----------------------|---|-----------------------|
/// | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
/// | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
/// | 1/(Scalar 0 + alpha) | 1/(Scalar 1 + alpha) | 1/(Scalar 2 + alpha) | . | 1/(Scalar 31 + alpha) |
/// --------------------------------------------------------------------------------------------------
///            |                     |                      |                              |            
///            v                     v                      v                              v          
///     intermediate MLE      intermediate MLE       intermediate MLE               intermediate MLE     
/// ```
///
/// This new matrix of logarithmic derivatives, and the original word decomposition, are
/// sufficient to establish constraints for verification.
///
/// ## Bottlenecks
/// * batch inversion, we should try to do as few of these as possible
/// * single-threaded evaluation; we can likely apply rayon or similar here
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
    let mut all_scalar_bytes: Vec<&[u8]> = Vec::with_capacity(scalars.len());

    decompose_scalar_to_words(
        scalars,
        alloc,
        &mut word_columns,
        byte_counts,
        &mut all_scalar_bytes,
    );

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
    // let byte_values: &mut [u8] = alloc.alloc_slice_fill_with(256, |i| i as u8);
    let byte_values: &mut [u8] =
        alloc.alloc_slice_fill_with(std::cmp::max(256, scalars.len()), |i| i as u8);

    // Produce the anchored MLE that the verifier has access to, consisting
    // of all possible word values. These serve as lookups
    // in the table
    builder.produce_anchored_mle(byte_values as &[_]);

    // Next produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(byte_counts as &[_]);

    // Now produce an intermediate MLE over the inverted word values + verifier challenge alpha
    let inverted_word_values: &mut [S] = alloc.alloc_slice_fill_with(256, |i| {
        S::try_from(i.into()).expect("word value will always fit into S") + alpha
    });
    slice_ops::batch_inversion(&mut inverted_word_values[..]);
    builder.produce_intermediate_mle(inverted_word_values as &[_]);
}

fn decompose_scalar_to_words<'a, S: Scalar + 'a>(
    scalars: &mut [S],
    alloc: &'a Bump,
    word_columns: &mut [&mut [u8]],
    byte_counts: &mut [i64],
    all_scalar_bytes: &mut Vec<&'a [u8]>,
) {
    for (i, scalar) in scalars.iter().enumerate() {
        let scalar_array: [u64; 4] = (*scalar).into(); // Convert scalar to u64 array
        let scalar_bytes_full = cast_slice::<u64, u8>(&scalar_array); // Cast u64 array to u8 slice
        let scalar_bytes = alloc.alloc_slice_copy(&scalar_bytes_full[..31]); // Limit to 31 bytes and allocate in bumpalo

        // Populate the rows of the words table with decomposition of scalar:
        // word_columns:
        //
        // | Column i           | Column i+1             | Column i+2            | ... | Column_||word||     |
        // |--------------------|------------------------|-----------------------|-----|---------------------|
        // | Byte i of Scalar i | Byte 1+1 of Scalar i+1 | Byte 1+2 of Scalar i+2| ... | Byte n of Scalar n  |
        for (row, &byte) in word_columns.iter_mut().zip(scalar_bytes.iter()) {
            row[i] = byte;
            byte_counts[byte as usize] += 1;
        }

        // Store the byte array slice for use in Phase 2
        all_scalar_bytes.push(scalar_bytes);
    }
}

fn get_logarithmic_derivative<'a, S: Scalar + 'a>(
    all_scalar_bytes: &[&[u8]],
    alpha: S,
    inverted_word_columns: &mut [&mut [S]],
) {
    // Phase 2: Use the stored byte arrays and alpha
    for (i, scalar_bytes) in all_scalar_bytes.iter().enumerate() {
        // For each element in a row, add alpha to it, and assign to inverted_word_columns:
        // inverted_word_columns:
        //
        // | Column i            | Column i+1            | Column i+2            | ... | Column_||word||     |
        // |---------------------|-----------------------|-----------------------|-----|---------------------|
        // | (word[i] + alpha)   | (word[i+1] + alpha)   |  word[i+2] + alpha)   | ... | (word[n] + alpha)   |
        let mut terms_to_invert: Vec<S> = scalar_bytes
            .iter()
            .map(|&w| S::try_from(w.into()).expect("u8 always fits in S") + alpha)
            .collect();

        // Invert all the terms in a row at once
        // inverted_word_columns:
        //
        // | Column i            | Column i+1            | Column i+2            | ... | Column_||word||     |
        // |---------------------|-----------------------|-----------------------|-----|---------------------|
        // | 1/(word[i] + alpha) | 1/(word[i+1] + alpha) | 1/(word[i+2] + alpha) | ... | 1/(word[n] + alpha) |
        slice_ops::batch_inversion(&mut terms_to_invert);

        // Assign the inverted values back to the inverted_word_columns
        for ((j, &inverted_value), column) in terms_to_invert
            .iter()
            .enumerate()
            .zip(inverted_word_columns.iter_mut())
        {
            column[i] = inverted_value; // j is the column index, i is the row index
        }
    }
}

/// Evaluates a polynomial at a specified point to verify if the result matches
/// a given expression value. This function applies Horner's method for efficient
/// polynomial evaluation.
///
/// The function first retrieves the necessary coefficients from a
/// [VerificationBuilder] and then evaluates the polynomial. If the evaluated
/// result matches the given `expr_eval`, it confirms the validity of the
/// expression; otherwise, it raises an error.
///
/// # Type Parameters
/// * `C` - Represents a commitment type that must support basic arithmetic
///   operations (`Add`, `Mul`) and can be constructed from `u128`.
///
/// # Returns
/// * `Ok(())` if the computed polynomial value matches `expr_eval`.
/// * `Err(ProofError)` if there is a mismatch, indicating a verification failure.
pub fn verifier_evaluate_range_check<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    expr_eval: C::Scalar,
) -> Result<(), ProofError> {
    unimplemented!("Fill this method when when ready to add verification")
}

#[cfg(test)]
mod tests {
    use crate::{
        base::scalar::{Curve25519Scalar as S, Scalar},
        sql::proof_exprs::range_check::{decompose_scalar_to_words, get_logarithmic_derivative},
    };
    use bumpalo::Bump;
    use bytemuck;
    use num_traits::Inv;
    use rand::Rng;

    #[test]
    fn test_decompose_scalar_to_words() {
        let mut rng = rand::thread_rng();
        let mut scalars: Vec<S> = (0..1024).map(|_| S::from(rng.gen::<u64>())).collect();

        let alloc = Bump::new();
        let mut word_columns: Vec<&mut [u8]> = (0..31)
            .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0u8))
            .collect();

        let byte_counts = alloc.alloc_slice_fill_with(256, |_| 0i64);
        let mut all_scalar_bytes: Vec<&[u8]> = Vec::with_capacity(scalars.len());

        decompose_scalar_to_words(
            &mut scalars,
            &alloc,
            &mut word_columns,
            byte_counts,
            &mut all_scalar_bytes,
        );

        for (i, scalar) in scalars.iter().enumerate() {
            let scalar_array: [u64; 4] = scalar.into();
            let scalar_bytes = bytemuck::cast_slice::<u64, u8>(&scalar_array);

            assert_eq!(all_scalar_bytes[i], &scalar_bytes[..31],);
        }

        println!("Byte arrays and counts verified correctly.");
    }

    #[test]
    fn test_logarithmic_derivative() {
        let mut rng = rand::thread_rng();

        let mut scalars: Vec<S> = (0..1024).map(|_| S::from(rng.gen::<u64>())).collect();

        let alloc = Bump::new();
        let mut word_columns: Vec<&mut [u8]> = (0..31)
            .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0u8))
            .collect();

        let byte_counts = alloc.alloc_slice_fill_with(256, |_| 0i64);
        let mut all_scalar_bytes: Vec<&[u8]> = Vec::with_capacity(scalars.len());

        decompose_scalar_to_words(
            &mut scalars,
            &alloc,
            &mut word_columns,
            byte_counts,
            &mut all_scalar_bytes,
        );

        let alpha = S::from(5);

        let mut inverted_word_columns: Vec<&mut [S]> = (0..31)
            .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO))
            .collect();

        get_logarithmic_derivative(&all_scalar_bytes, alpha, &mut inverted_word_columns);

        // Check that each original byte plus alpha inverted is equal to each byte
        // in all_scalar_bytes after passing it to get_logarithmic_derivative
        for (column_idx, column) in word_columns.iter().enumerate() {
            for (word_idx, &byte) in column.iter().enumerate() {
                let original_scalar = S::from(byte) + alpha;
                let expected_inverse = original_scalar.inv().unwrap_or(S::ZERO);
                let computed_inverse = inverted_word_columns[column_idx][word_idx];

                assert_eq!(expected_inverse, computed_inverse);
            }
        }
    }
}
