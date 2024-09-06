use crate::base::{scalar::Scalar, slice_ops};
use bytemuck::cast_slice;

// Decomposes a scalar to requisite words, additionally tracks the total
// number of occurences of each word for later use in the argument.
fn decompose_scalar_to_words<'a, S: Scalar + 'a>(
    scalars: &mut [S],
    word_columns: &mut [&mut [u8]],
    byte_counts: &mut [u64],
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
        let mut terms_to_invert: Vec<S> = byte_column
            .iter()
            .map(|&w| S::try_from(w.into()).unwrap() + alpha)
            .collect();

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

        let expected_byte_counts = [
            53, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 8,
        ];

        assert_eq!(word_columns[..16], expected_word_columns);
        assert_eq!(byte_counts, expected_byte_counts);
    }

    #[test]
    fn we_can_obtain_logarithmic_derivative_from_small_scalar() {
        let scalars: Vec<S> = [1, 2, 3, 255, 256, 257].iter().map(S::from).collect();
        let mut word_columns = vec![vec![0; scalars.len()]; 31];

        word_columns[0] = [1, 2, 3, 255, 0, 1].to_vec();
        word_columns[1] = [0, 0, 0, 0, 1, 1].to_vec();

        let word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();

        let alpha = S::from(5);

        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut inverted_word_columns_plus_alpha_as_slices: Vec<&mut [S]> =
            inverted_word_columns_plus_alpha
                .iter_mut()
                .map(|col| col.as_mut_slice())
                .collect();

        get_logarithmic_derivative(
            &word_slices,
            alpha,
            &mut inverted_word_columns_plus_alpha_as_slices,
        );

        let expected_column: Vec<S> = [1, 2, 3, 255, 0, 1]
            .iter()
            .map(|w| (S::from(*w) + alpha).inv().unwrap_or(S::ZERO))
            .collect();

        assert_eq!(
            inverted_word_columns_plus_alpha_as_slices[0],
            expected_column.as_slice()
        );

        let expected_column: Vec<S> = [0, 0, 0, 0, 1, 1]
            .iter()
            .map(|w| (S::from(*w) + alpha).inv().unwrap_or(S::ZERO))
            .collect();

        assert_eq!(
            inverted_word_columns_plus_alpha_as_slices[1],
            expected_column.as_slice()
        );

        // expected_word_columns[2..] is filled with 0s.
        let expected_column: Vec<S> = [0, 0, 0, 0, 0, 0]
            .iter()
            .map(|w| (S::from(*w) + alpha).inv().unwrap_or(S::ZERO))
            .collect();

        assert_eq!(
            inverted_word_columns_plus_alpha_as_slices[2],
            expected_column.as_slice()
        );
    }

    #[test]
    fn we_can_obtain_logarithmic_derivative_from_large_scalar() {
        // let scalars: Vec<S> = [u64::MAX, u64::MAX].iter().map(S::from).collect();
        let scalars: Vec<S> = [0xFF, 0xFF].iter().map(S::from).collect();
        let mut word_columns = vec![vec![0; scalars.len()]; 31];

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

        let word_slices: Vec<&mut [u8]> = word_columns.iter_mut().map(|c| &mut c[..]).collect();

        let alpha = S::from(5);

        let mut inverted_word_columns_plus_alpha: Vec<Vec<S>> =
            vec![vec![S::ZERO; scalars.len()]; 31];

        // Convert Vec<Vec<S>> into Vec<&mut [S]> for use in get_logarithmic_derivative
        let mut inverted_word_columns_plus_alpha_as_slices: Vec<&mut [S]> =
            inverted_word_columns_plus_alpha
                .iter_mut()
                .map(|col| col.as_mut_slice())
                .collect();

        get_logarithmic_derivative(
            &word_slices,
            alpha,
            &mut inverted_word_columns_plus_alpha_as_slices,
        );

        // Expected values defined over a larger array
        let expected_data = [
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

        // Invert the expected data
        let expected_columns: Vec<Vec<S>> = expected_data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|&w| (S::from(w) + alpha).inv().unwrap_or(S::ZERO))
                    .collect()
            })
            .collect();

        assert_eq!(inverted_word_columns_plus_alpha_as_slices, expected_columns);
    }
}
