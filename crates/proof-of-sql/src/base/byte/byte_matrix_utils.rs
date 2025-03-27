use super::ByteDistribution;
use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shr;

#[expect(clippy::missing_panics_doc)]
pub fn compute_varying_byte_matrix<S: Scalar>(
    column_data: &[impl Copy + Into<S>],
    dist: &ByteDistribution,
) -> impl Iterator<Item = Vec<u8>> + Clone {
    dist.varying_byte_indices()
        .map(|start_index| {
            column_data
                .iter()
                .map(move |row| {
                    let bit_mask = make_bit_mask((*row).into());
                    let shifted_byte: u8 = (bit_mask.shr(start_index) & U256::from(255u8))
                        .try_into()
                        .unwrap();
                    shifted_byte
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn get_word_counts(alloc: &Bump, columns: impl Iterator<Item = Vec<u8>>) -> &[i64] {
    let word_counts = alloc.alloc_slice_fill_copy(256, 0i64);
    for byte in columns.into_iter().flatten() {
        word_counts[byte as usize] += 1;
    }
    word_counts
}

#[cfg(test)]
mod tests {
    use crate::base::{
        byte::{
            byte_matrix_utils::{compute_varying_byte_matrix, get_word_counts},
            ByteDistribution,
        },
        scalar::{test_scalar::TestScalar, Scalar},
    };
    use bumpalo::Bump;

    #[test]
    fn we_can_compute_varying_byte_matrix_for_small_scalars() {
        let scalars: Vec<TestScalar> = [1, 2, 3, 255, 256, 257]
            .iter()
            .map(TestScalar::from)
            .collect();
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&scalars);
        let varying_columns =
            compute_varying_byte_matrix::<TestScalar>(&scalars, &byte_distribution)
                .collect::<Vec<_>>();
        let expected_word_columns = vec![vec![1, 2, 3, 255, 0, 1], vec![0, 0, 0, 0, 1, 1]];
        assert_eq!(varying_columns, expected_word_columns);
    }

    #[test]
    fn we_can_get_word_count_for_small_scalars() {
        let alloc = Bump::new();
        let words = [vec![1u8, 2, 3, 255, 0, 1], vec![0, 0, 0, 0, 1, 1]].into_iter();
        let word_counts = get_word_counts(&alloc, words);
        let mut expected_word_counts = vec![0; 256];
        expected_word_counts[0] = 5;
        expected_word_counts[1] = 4;
        expected_word_counts[2] = 1;
        expected_word_counts[3] = 1;
        // expected_byte_counts[4..255] is filled with 0s.
        expected_word_counts[255] = 1;

        assert_eq!(word_counts, expected_word_counts);
    }

    #[test]
    fn we_can_compute_varying_byte_matrix_for_large_scalars() {
        let scalars = vec![
            TestScalar::MAX_SIGNED,
            TestScalar::from(u64::MAX),
            -TestScalar::ONE,
        ];
        let byte_distribution = ByteDistribution::new::<TestScalar, _>(&scalars);
        let varying_columns =
            compute_varying_byte_matrix::<TestScalar>(&scalars, &byte_distribution)
                .collect::<Vec<_>>();

        let expected_word_columns = vec![
            [246, 255, 255],
            [233, 255, 255],
            [122, 255, 255],
            [46, 255, 255],
            [141, 255, 255],
            [49, 255, 255],
            [9, 255, 255],
            [44, 255, 255],
            [107, 0, 255],
            [206, 0, 255],
            [123, 0, 255],
            [81, 0, 255],
            [239, 0, 255],
            [124, 0, 255],
            [111, 0, 255],
            [10, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [136, 128, 127],
        ];

        assert_eq!(varying_columns, expected_word_columns);
    }
}
