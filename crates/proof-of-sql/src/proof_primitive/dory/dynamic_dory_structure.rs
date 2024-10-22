//! This module gives some utility functions for determining the position of data within the dynamic dory matrix
//!
//! In general, the data is filled in such a way that the new data is always in the last row, and the row size
//! (and consequently, the matrix size) is strictly increasing.
//! Aside from the first 3 rows, the pattern is to have 3\*2^n rows of length 4\*2^n.
//! In particular this means that a 2^n by 2^n matrix contains exactly 2^(2\*n-1) data points when n>0.
//!
//! This structure allows for a multilinear point evaluation of the associated MLE to be represented as a
//! relatively simple matrix product.
//!
//! Concretely, if the data being committed to is 0, 1, 2, 3, etc., the matrix is filled as shown below.
//!
//! ```text
//!   0
//!   _,   1
//!   2,   3
//!   4,   5,   6,   7
//!   8,   9,  10,  11
//!  12,  13,  14,  15
//!  16,  17,  18,  19,  20,  21,  22,  23
//!  24,  25,  26,  27,  28,  29,  30,  31
//!  32,  33,  34,  35,  36,  37,  38,  39
//!  40,  41,  42,  43,  44,  45,  46,  47
//!  48,  49,  50,  51,  52,  53,  54,  55
//!  56,  57,  58,  59,  60,  61,  62,  63
//!  64,  65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,  77,  78,  79
//!  80,  81,  82,  83,  84,  85,  86,  87,  88,  89,  90,  91,  92,  93,  94,  95
//!  96,  97,  98,  99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111
//! 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127
//! ...
//! ```

/// Returns the full width of a row in the matrix.
///
/// Note: when row = 1, this correctly returns 2, even though no data belongs at position 0.
pub(crate) const fn full_width_of_row(row: usize) -> usize {
    ((2 * row + 4) / 3).next_power_of_two()
}

/// Returns the index that belongs in the first column in a particular row.
///
/// Note: when row = 1, this correctly returns 0, even though no data belongs there.
#[cfg(test)]
pub(crate) const fn row_start_index(row: usize) -> usize {
    let width_of_row = full_width_of_row(row);
    width_of_row * (row - width_of_row / 2)
}

/// Returns the (row, column) in the matrix where the data with the given index belongs.
pub(crate) const fn row_and_column_from_index(index: usize) -> (usize, usize) {
    let width_of_row = 1 << (((2 * index + 1).ilog2() + 1) / 2);
    let row = index / width_of_row + width_of_row / 2;
    let column = index % width_of_row;
    (row, column)
}

/// Returns the index of data where the (row, column) belongs.
pub(crate) fn index_from_row_and_column(row: usize, column: usize) -> Option<usize> {
    let width_of_row = full_width_of_row(row);
    (column < width_of_row && (row, column) != (1, 0))
        .then_some(width_of_row * (row - width_of_row / 2) + column)
}

/// Returns a matrix size, (height, width), that can hold the given number of data points being committed with respect to an offset.
///
/// Note: when `data_len = 0` and `offset = 0`, this function returns an empty matrix with size (0, 0).
pub(crate) const fn matrix_size(data_len: usize, offset: usize) -> (usize, usize) {
    if data_len == 0 && offset == 0 {
        return (0, 0);
    }

    let (last_row, _) = row_and_column_from_index(offset + data_len - 1);
    let width_of_last_row = full_width_of_row(last_row);
    (last_row + 1, width_of_last_row)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Creates a 2^nu by 2^nu matrix that is partially filled with the indexes that belong in each position.
    fn create_position_matrix(nu: usize) -> Vec<Vec<Option<usize>>> {
        let max_index = 1 << (2 * nu - 1);
        let mut matrix = vec![];
        for i in 0..max_index {
            let (row, column) = row_and_column_from_index(i);
            if matrix.len() <= row {
                matrix.resize_with(row + 1, Vec::new);
            }
            if matrix[row].len() <= column {
                matrix[row].resize(column + 1, None);
            }
            matrix[row][column] = Some(i);
        }
        matrix
    }
    #[test]
    fn we_can_compute_positions_from_indexes() {
        assert_eq!(
            create_position_matrix(2),
            vec![
                vec![Some(0)],                            // length 1
                vec![None, Some(1)],                      // length "1"
                vec![Some(2), Some(3)],                   // length 2
                vec![Some(4), Some(5), Some(6), Some(7)], // length 4
            ],
        );
        assert_eq!(
            create_position_matrix(4),
            vec![
                vec![Some(0)],                                // length 1
                vec![None, Some(1)],                          // length "1"
                vec![Some(2), Some(3)],                       // length 2
                vec![Some(4), Some(5), Some(6), Some(7)],     // length 4
                vec![Some(8), Some(9), Some(10), Some(11)],   // length 4
                vec![Some(12), Some(13), Some(14), Some(15)], // length 4
                (16..=23).map(Some).collect(),                // length 8
                (24..=31).map(Some).collect(),                // length 8
                (32..=39).map(Some).collect(),                // length 8
                (40..=47).map(Some).collect(),                // length 8
                (48..=55).map(Some).collect(),                // length 8
                (56..=63).map(Some).collect(),                // length 8
                (64..=79).map(Some).collect(),                // length 16
                (80..=95).map(Some).collect(),                // length 16
                (96..=111).map(Some).collect(),               // length 16
                (112..=127).map(Some).collect()               // length 16
            ],
        );
    }
    #[test]
    fn we_can_fill_matrix_with_no_collisions_and_correct_size_and_row_starts() {
        for nu in 1..10 {
            let num_vars = nu * 2 - 1;
            let matrix = create_position_matrix(nu);
            // Every position should be unique.
            assert_eq!(
                matrix.iter().flatten().filter(|&x| x.is_some()).count(),
                1 << num_vars
            );
            // The matrix should have 2^nu rows.
            assert_eq!(matrix.len(), 1 << nu);
            // The matrix should have 2^nu columns.
            assert_eq!(matrix.iter().map(Vec::len).max().unwrap(), 1 << nu);
            for (index, row) in matrix.iter().enumerate() {
                // The start of each row should match with `row_start_index`.
                assert_eq!(
                    row_start_index(index),
                    match index {
                        1 => 0, // Row 1 starts at 0, even though nothing is put in the 0th position. This is because the 0th index is at position (0, 0)
                        _ => row[0]
                            .expect("Every row except 1 should have a value in the 0th position."),
                    }
                );
            }
        }
    }
    #[test]
    fn we_can_find_the_full_width_of_row() {
        // This corresponds to a matrix with 2^(N+1) rows.
        let N = 20;
        let mut expected_widths = Vec::new();

        // First three rows are defined by the dynamic Dory structure.
        expected_widths.extend(std::iter::repeat(1).take(1));
        expected_widths.extend(std::iter::repeat(2).take(2));

        // The rest of the rows are defined by the pattern 3*2^n rows of length 4*2^n.
        for n in 0..N {
            let repeat_count = 3 * 2_usize.pow(n);
            let value = 4 * 2_usize.pow(n);
            expected_widths.extend(std::iter::repeat(value).take(repeat_count));
        }

        // Verify the widths.
        for (row, width) in expected_widths.iter().enumerate() {
            let width_of_row = full_width_of_row(row);
            assert_eq!(
                width_of_row, *width,
                "row: {row} does not produce expected width"
            );
        }
    }
    #[test]
    fn we_can_produce_the_correct_matrix_size() {
        // NOTE: returned tuple is (height, width).
        assert_eq!(matrix_size(0, 0), (0, 0));
        assert_eq!(matrix_size(1, 0), (1, 1));
        assert_eq!(matrix_size(2, 0), (2, 2));
        assert_eq!(matrix_size(3, 0), (3, 2));
        assert_eq!(matrix_size(4, 0), (3, 2));
        assert_eq!(matrix_size(5, 0), (4, 4));
        assert_eq!(matrix_size(6, 0), (4, 4));
        assert_eq!(matrix_size(7, 0), (4, 4));
        assert_eq!(matrix_size(8, 0), (4, 4));
        assert_eq!(matrix_size(9, 0), (5, 4));
        assert_eq!(matrix_size(10, 0), (5, 4));
        assert_eq!(matrix_size(11, 0), (5, 4));
        assert_eq!(matrix_size(12, 0), (5, 4));
        assert_eq!(matrix_size(13, 0), (6, 4));
        assert_eq!(matrix_size(14, 0), (6, 4));
        assert_eq!(matrix_size(15, 0), (6, 4));
        assert_eq!(matrix_size(16, 0), (6, 4));
        assert_eq!(matrix_size(17, 0), (7, 8));

        assert_eq!(matrix_size(64, 0), (12, 8));
        assert_eq!(matrix_size(71, 0), (13, 16));
        assert_eq!(matrix_size(81, 0), (14, 16));
        assert_eq!(matrix_size(98, 0), (15, 16));
        assert_eq!(matrix_size(115, 0), (16, 16));
    }
    #[test]
    fn we_can_produce_the_correct_matrix_size_with_offset() {
        // NOTE: returned tuple is (height, width).
        assert_eq!(matrix_size(0, 0), (0, 0));
        assert_eq!(matrix_size(0, 1), (1, 1));
        assert_eq!(matrix_size(0, 2), (2, 2));
        assert_eq!(matrix_size(1, 1), (2, 2));
        assert_eq!(matrix_size(1, 2), (3, 2));
        assert_eq!(matrix_size(1, 3), (3, 2));
        assert_eq!(matrix_size(1, 4), (4, 4));
        assert_eq!(matrix_size(1, 5), (4, 4));
        assert_eq!(matrix_size(1, 6), (4, 4));
        assert_eq!(matrix_size(1, 7), (4, 4));
        assert_eq!(matrix_size(1, 8), (5, 4));
        assert_eq!(matrix_size(1, 9), (5, 4));
        assert_eq!(matrix_size(1, 10), (5, 4));
        assert_eq!(matrix_size(1, 11), (5, 4));
        assert_eq!(matrix_size(1, 12), (6, 4));
        assert_eq!(matrix_size(1, 13), (6, 4));
        assert_eq!(matrix_size(1, 14), (6, 4));
        assert_eq!(matrix_size(1, 15), (6, 4));
        assert_eq!(matrix_size(1, 16), (7, 8));

        assert_eq!(matrix_size(1, 63), (12, 8));
        assert_eq!(matrix_size(1, 70), (13, 16));
        assert_eq!(matrix_size(1, 80), (14, 16));
        assert_eq!(matrix_size(1, 97), (15, 16));
        assert_eq!(matrix_size(1, 114), (16, 16));
    }
    #[test]
    fn we_can_find_the_index_for_row_column_pairs() {
        use std::collections::HashSet;

        const MAX_INDEX: usize = 1 << 16;
        let mut valid_pairs = HashSet::new();

        // Collect all valid (row, column) pairs
        for i in 0..MAX_INDEX {
            let (row, column) = row_and_column_from_index(i);
            valid_pairs.insert((row, column));
        }

        let (max_row, max_column) = valid_pairs
            .iter()
            .fold((0, 0), |(max_row, max_column), &(row, column)| {
                (max_row.max(row), max_column.max(column))
            });

        // Check that all valid pairs are valid and all invalid pairs are invalid
        for row in 0..max_row {
            for column in 0..max_column {
                let index = index_from_row_and_column(row, column);
                if valid_pairs.contains(&(row, column)) {
                    assert!(
                        index.is_some(),
                        "Valid pair ({row}, {column}) generated no index"
                    );
                } else {
                    assert!(
                        index.is_none(),
                        "Invalid pair ({row}, {column}) generated a valid index"
                    );
                }
            }
        }
    }
}
