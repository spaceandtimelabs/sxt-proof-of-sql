//! This module gives some utility functions for determining the position of data within the dynamic dory/hyrax matrix
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
#[cfg(any(test, not(feature = "blitzar")))]
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
