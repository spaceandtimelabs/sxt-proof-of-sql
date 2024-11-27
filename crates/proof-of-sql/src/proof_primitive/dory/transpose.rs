use crate::proof_primitive::dory::offset_to_bytes::OffsetToBytes;
use alloc::{vec, vec::Vec};

#[tracing::instrument(name = "transpose_for_fixed_msm (gpu)", level = "debug", skip_all)]
pub fn transpose_for_fixed_msm<const LEN: usize, T: OffsetToBytes<LEN>>(
    column: &[T],
    offset: usize,
    rows: usize,
    cols: usize,
    data_size: usize,
) -> Vec<u8> {
    let total_length_bytes = data_size * rows * cols;
    let mut transpose = vec![0_u8; total_length_bytes];
    for n in offset..(column.len() + offset) {
        let i = n / cols;
        let j = n % cols;
        let t_idx = (j * rows + i) * data_size;
        let p_idx = (i * cols + j) - offset;

        transpose[t_idx..t_idx + data_size]
            .copy_from_slice(column[p_idx].offset_to_bytes().as_slice());
    }
    transpose
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;
    use zerocopy::AsBytes;

    #[test]
    fn we_can_transpose_empty_column() {
        type T = u64;
        let column: Vec<T> = vec![];
        let offset = 0;
        let rows = 0;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);
        assert!(transpose.is_empty());
    }

    #[test]
    fn we_can_transpose_u64_column() {
        type T = u64;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_column_with_offset() {
        type T = u64;
        let column: Vec<T> = vec![1, 2, 3];
        let offset = 2;
        let rows = 2;
        let cols = 3;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset + 1);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], 0_u64.as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[1].as_bytes());
        assert_eq!(&transpose[2 * data_size..3 * data_size], 0_u64.as_bytes());
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[2].as_bytes()
        );
        assert_eq!(
            &transpose[4 * data_size..5 * data_size],
            column[0].as_bytes()
        );
        assert_eq!(&transpose[5 * data_size..6 * data_size], 0_u64.as_bytes());
    }

    #[test]
    fn we_can_transpose_boolean_column_with_offset() {
        type T = bool;
        let column: Vec<T> = vec![true, false, true];
        let offset = 1;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], 0_u8.as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[1].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[0].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[2].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_i64_column() {
        type T = i64;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(
            &transpose[0..data_size],
            column[0].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[data_size..2 * data_size],
            column[2].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].wrapping_sub(T::MIN).as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_i128_column() {
        type T = i128;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(
            &transpose[0..data_size],
            column[0].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[data_size..2 * data_size],
            column[2].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].wrapping_sub(T::MIN).as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_array_column() {
        type T = [u64; 4];
        let column: Vec<T> = vec![[0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_array_column_update() {
        type T = [u64; 4];
        let column: Vec<T> = vec![[0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }
}
