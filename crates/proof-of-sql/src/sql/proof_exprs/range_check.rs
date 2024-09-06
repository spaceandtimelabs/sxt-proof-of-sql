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
/// the same byte position across all scalars:
///
/// ```text
/// | Column 0           | Column 1           | Column 2           | ... | Column 30           |  
/// |--------------------|--------------------|--------------------|-----|---------------------|  
/// | Byte 0 of Scalar 0 | Byte 1 of Scalar 0 | Byte 2 of Scalar 0 | ... | Byte 30 of Scalar 0 |  
/// | Byte 0 of Scalar 1 | Byte 1 of Scalar 1 | Byte 2 of Scalar 1 | ... | Byte 30 of Scalar 1 |  
/// | Byte 0 of Scalar 2 | Byte 1 of Scalar 2 | Byte 2 of Scalar 2 | ... | Byte 30 of Scalar 2 |  
/// ```
/// After constructing this matrix, each byte column is used to produce an intermediate MLE.
pub fn prover_evaluate_range_check<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    scalars: &mut [S],
    alloc: &'a Bump,
) {
    // Create 31 columns, each will collect the corresponding byte from all scalars.
    // 31 because a scalar will only ever have 248 bits of data set.
    let mut columns: Vec<&mut [u8]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| 0))
        .collect();

    // Initialize a vector to count occurrences of each byte (0-255) using field elements `S`.
    // The vector has 256 elements padded with zeros to match the length of the word columns
    // and are each initialized to the zero element of `S`.
    // TODO: this should equal the length of the column of scalars
    let byte_counts: &mut [S] = alloc.alloc_slice_fill_with(256, |_| S::zero());

    // Iterate through scalars and fill columns
    for (i, scalar) in scalars.iter().enumerate() {
        // Convert scalar into an array of u64, then into byte-sized words
        let scalar_array: [u64; 4] = (*scalar).into();
        let scalar_bytes = bytemuck::cast_slice::<u64, u8>(&scalar_array); // Safer casting using bytemuck

        // Populate columns and update byte counts
        for (col, &byte) in columns.iter_mut().zip(scalar_bytes.iter()) {
            col[i] = byte;

            // Update the byte count in the corresponding position
            byte_counts[byte as usize] += S::one(); // Increment the count of the byte value
        }
    }

    // Allocate and initialize byte_values to represent each possible byte as a scalar directly
    let byte_values: &mut [S] =
        alloc.alloc_slice_fill_with(256, |i| S::try_from(i.into()).unwrap());

    // 1. Produce an MLE over each column of words
    for column in columns {
        builder.produce_intermediate_mle(column as &[u8]);
    }

    // 2. Produce the anchored MLE that the verifier has access to, consisting
    // of all possible word values. These serve as values to lookup
    // in the lookup table
    builder.produce_anchored_mle(byte_values as &[S]);

    // 3. Next produce an MLE over the counts of each word value
    builder.produce_intermediate_mle(byte_counts as &[S]);

    // Invert the scalars, and get the inverted words.
    // This modifies the column in place.
    slice_ops::batch_inversion(&mut scalars[..]);
    let mut inverted_word_columns: Vec<&mut [S]> = (0..31)
        .map(|_| alloc.alloc_slice_fill_with(scalars.len(), |_| S::ZERO))
        .collect();

    // Get the alpha challenge from the verifier
    let alpha = builder.consume_post_result_challenge();

    // Iterate through the inverted scalars and fill columns
    for (i, inverted_scalar) in scalars.iter().enumerate() {
        let inverted_scalar_array: [u64; 4] = (*inverted_scalar).into();
        let inverted_scalar_words = bytemuck::cast_slice::<u64, u8>(&inverted_scalar_array);

        // Allocate and initialize row for each inverted scalar processing
        let row: &mut [S] = alloc.alloc_slice_fill_with(inverted_scalar_words.len(), |_| S::zero());

        for ((col, &inverted_word), row_entry) in inverted_word_columns
            .iter_mut()
            .zip(inverted_scalar_words.iter())
            .zip(row.iter_mut())
        {
            // Convert a word into a scalar so that we can perform arithmetic on it
            let value =
                S::try_from(inverted_word.into()).expect("u8 will always fit in scalar") + alpha;
            col[i] = value;
            *row_entry = value;
        }
        builder.produce_intermediate_mle(&*row);
    }

    // Now produce an intermediate MLE over the inverted word values + verifier challenge alpha
    let inverted_word_values: &mut [S] =
        alloc.alloc_slice_fill_with(256, |i| S::try_from(i.into()).unwrap() + alpha);
    slice_ops::batch_inversion(&mut inverted_word_values[..]);
    builder.produce_anchored_mle(inverted_word_values as &[S]);
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
