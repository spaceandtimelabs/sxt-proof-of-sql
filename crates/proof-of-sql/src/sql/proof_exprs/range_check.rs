use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar, slice_ops},
    sql::proof::{ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;

/// Decomposes a column of scalars into a matrix of words, so that each word column can be
/// used to produce an intermediate MLE. Produces intermediate MLEs for:
/// * each column of words
/// * the count of how many times each word occurs
///
/// And anchored MLEs for:
/// * all possible byte values
///
/// ## Word-sized decomposition:
///
/// Each row represents the byte decomposition of a scalar, and each column contains the bytes from
/// the same byte position across all scalars. First, we produce this word-wise decomposition:
///
/// ```text
/// | Column 0           | Column 1           | Column 2           | ... | Column 31           |  
/// |--------------------|--------------------|--------------------|-----|---------------------|  
/// | Byte 0 of Scalar 0 | Byte 1 of Scalar 0 | Byte 2 of Scalar 0 | ... | Byte 31 of Scalar 0 |  
/// | Byte 0 of Scalar 1 | Byte 1 of Scalar 1 | Byte 2 of Scalar 1 | ... | Byte 31 of Scalar 1 |  
/// | Byte 0 of Scalar 2 | Byte 1 of Scalar 2 | Byte 2 of Scalar 2 | ... | Byte 31 of Scalar 2 |  
/// ```
///
/// The next step is to compute intermediate MLEs over the word columns:
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
/// populated. An anchored MLE is produced over this column, since the verifier knows range
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
pub fn prover_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    scalars: &mut [S],
    alloc: &'a Bump,
) {
    // Create 31 columns, each will collect the corresponding byte from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut words: Vec<&mut [u8]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0))
        .collect();

    // Allocate space for the eventual inverted word columns
    let mut inverted_word_columns: Vec<&mut [S]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO))
        .collect();

    // Initialize a vector to count occurrences of each byte (0-255).
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // TODO: this should equal the length of the column of scalars
    let byte_counts: &mut [i64] = alloc.alloc_slice_fill_with(256, |_| 0);

    // Get the alpha challenge from the verifier
    let alpha = builder.consume_post_result_challenge();

    // Iterate through scalars and fill columns
    for (i, scalar) in scalars.iter().enumerate() {
        // Convert scalar into an array of u64, then break into words
        let scalar_array: [u64; 4] = (*scalar).into();
        let scalar_bytes = &bytemuck::cast_slice::<u64, u8>(&scalar_array)[..31]; // Limit to 31 bytes

        // Populate the rows of the words table with decomposition of scalar:
        //
        // | Column i           | Column i+1             | Column i+2            | ... | Column_||word||     |
        // |--------------------|------------------------|-----------------------|-----|---------------------|
        // | Byte i of Scalar i | Byte 1+1 of Scalar i+1 | Byte 1+2 of Scalar i+2| ... | Byte n of Scalar n  |
        for (row, &byte) in words.iter_mut().zip(scalar_bytes.iter()) {
            row[i] = byte;
            byte_counts[byte as usize] += 1; // Also count how many times we see this word
        }

        // Convert each word to scalar so we can perform requisite arithmetic on it
        let inverted_words: Vec<S> = scalar_bytes
            .iter()
            .map(|w| S::try_from((*w).into()).expect("u8 always fits in S"))
            .collect();

        // For each element in a row, add alpha to it, and assign to inverted_word_columns:
        //
        // | Column i            | Column i+1            | Column i+2            | ... | Column_||word||     |
        // |---------------------|-----------------------|-----------------------|-----|---------------------|
        // | 1/(word[i] + alpha) | 1/(word[i+1] + alpha) | 1/(word[i+2] + alpha) | ... | 1/(word[n] + alpha) |
        for (j, &word_scalar) in inverted_words.iter().enumerate() {
            // Add the verifier challenge to the word, then invert the sum in the scalar field
            let value: S = (word_scalar + alpha).inv().unwrap_or(S::ZERO);
            inverted_word_columns[j][i] = value; // j is column index, i is row index
        }
    }

    // Produce an MLE over each column of words
    for word_column in words {
        builder.produce_intermediate_mle(word_column as &[_]);
    }

    // Produce an MLE over each (word + alpha)^-1 column
    for inverted_word_column in inverted_word_columns {
        builder.produce_intermediate_mle(inverted_word_column as &[_]);
    }

    // Allocate and initialize byte_values to represent the range of possible word values
    // from 0 to 255.
    let byte_values: &mut [u8] = alloc.alloc_slice_fill_with(256, |i| i as u8);

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
    use crate::base::{
        scalar::{Curve25519Scalar as S, Scalar},
        slice_ops,
    };
    use bytemuck;

    #[test]
    fn test_scalar_transformation_and_inversion() {
        // Define a test scalar
        let scalar = S::from(u64::MAX);

        // Convert the scalar into an array of u64
        let scalar_array: [u64; 4] = scalar.into();

        // Convert the u64 array into a byte array
        let scalar_bytes = bytemuck::cast_slice::<u64, u8>(&scalar_array);

        // Assert the bytes are correct (as per previous tests)
        let expected_bytes = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // bytes of scalar
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding zeros
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(
            scalar_bytes, expected_bytes,
            "The byte transformation did not match the expected output."
        );

        // Set up for batch inversion
        let mut scalars = [scalar]; // Array containing the scalar to invert
        slice_ops::batch_inversion(&mut scalars);

        // After batch inversion, check the scalar to ensure it was modified
        let inverted_scalar = scalars[0];

        // Multiplication of the original scalar and its inverse
        let result = scalar * inverted_scalar;

        // Check if scalar * inverse - 1 is zero
        assert_eq!(result - S::ONE, S::ZERO);
    }
}
