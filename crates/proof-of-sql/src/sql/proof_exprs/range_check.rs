use crate::base::{scalar::Scalar, slice_ops};
use bumpalo::Bump;
use bytemuck::cast_slice;

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
