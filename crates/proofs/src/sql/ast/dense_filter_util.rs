use crate::base::{database::Column, scalar::ArkScalar, slice_ops};
use bumpalo::Bump;
use num_traits::One;

/// This function takes a selection vector and a set of columns and returns a
/// new set of columns that only contains the selected rows. The function
/// panics if the selection vector is a different length than the columns.
///
/// The function returns a tuple of the filtered columns and the number of
/// rows in the filtered columns.
pub fn filter_columns<'a>(
    alloc: &'a Bump,
    columns: &[Column<'a>],
    selection: &[bool],
) -> (Vec<Column<'a>>, usize) {
    let indexes: Vec<_> = selection
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| i)
        .collect();

    (
        Vec::from_iter(columns.iter().map(|column| match column {
            Column::BigInt(col) => {
                assert_eq!(col.len(), selection.len());
                Column::BigInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
            }
            Column::Int128(col) => {
                assert_eq!(col.len(), selection.len());
                Column::Int128(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
            }
            Column::VarChar((col, scals)) => {
                assert_eq!(col.len(), selection.len());
                assert_eq!(scals.len(), selection.len());
                Column::VarChar((
                    alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
                    alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
                ))
            }
            #[cfg(test)]
            Column::Scalar(col) => {
                assert_eq!(col.len(), selection.len());
                Column::Scalar(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
            }
        })),
        indexes.len(),
    )
}

/// This function takes a set of columns and returns a slice of scalars that
/// is the result of folding the columns.
///
/// The result `fold` satisfies
/// `fold[i] = alpha + sum (beta^j * columns[j][i]) for j in 0..columns.len()`
/// where each column is padded with 0s as needed.
///
/// Note: this means that `alpha` is the "default" value for the fold. (i.e. if `n` is larger than the longest column, the result will be padded with `alpha`.)
///
/// This is similar to applying `fold_vals` on each row with `one_val` set to `1`.
pub fn fold_columns<'a>(
    alloc: &'a Bump,
    alpha: ArkScalar,
    beta: ArkScalar,
    columns: &[Column],
    n: usize,
) -> &'a mut [ArkScalar] {
    let fold = alloc.alloc_slice_fill_copy(n, alpha);
    let mut multiplier = ArkScalar::one();
    for col in columns.iter() {
        match col {
            Column::BigInt(c) => slice_ops::mul_add_assign(fold, multiplier, c),
            Column::VarChar((_, c)) => slice_ops::mul_add_assign(fold, multiplier, c),
            Column::Int128(c) => slice_ops::mul_add_assign(fold, multiplier, c),
            #[cfg(test)]
            Column::Scalar(c) => slice_ops::mul_add_assign(fold, multiplier, c),
        }
        multiplier *= beta;
    }
    fold
}

/// This function takes a set of values and returns a scalar that is the
/// result of folding the values.
///
/// The result `fold` satisfies
/// `fold = alpha * one_val + sum (beta^j * vals[j]) for j in 0..vals.len()`
pub fn fold_vals(
    alpha: ArkScalar,
    beta: ArkScalar,
    vals: impl IntoIterator<Item = ArkScalar>,
    one_val: ArkScalar,
) -> ArkScalar {
    let mut fold = alpha * one_val;
    let mut multiplier = ArkScalar::one();
    for val in vals {
        fold += multiplier * val;
        multiplier *= beta;
    }
    fold
}
