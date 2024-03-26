use crate::base::{database::Column, polynomial::MultilinearExtension, scalar::Curve25519Scalar};
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
    columns: &[Column<'a, Curve25519Scalar>],
    selection: &[bool],
) -> (Vec<Column<'a, Curve25519Scalar>>, usize) {
    for col in columns {
        assert_eq!(col.len(), selection.len());
    }
    let indexes: Vec<_> = selection
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| i)
        .collect();
    let result_length = indexes.len();
    let filtered_result = Vec::from_iter(
        columns
            .iter()
            .map(|column| filter_column_by_index(alloc, column, &indexes)),
    );
    (filtered_result, result_length)
}
/// This function takes an index vector and a `Column` and returns a
/// new set of columns that only contains the selected indexes. It is assumed that
/// the indexes are valid.
pub fn filter_column_by_index<'a>(
    alloc: &'a Bump,
    column: &Column<'a, Curve25519Scalar>,
    indexes: &[usize],
) -> Column<'a, Curve25519Scalar> {
    match column {
        Column::BigInt(col) => {
            Column::BigInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Int128(col) => {
            Column::Int128(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::VarChar((col, scals)) => Column::VarChar((
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
        )),
        Column::Scalar(col) => {
            Column::Scalar(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Decimal75(precision, scale, col) => Column::Decimal75(
            *precision,
            *scale,
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
        ),
    }
}

/// This function takes a set of columns and fold it into a slice of scalars.
///
/// The result `res` is updated with
/// `res[i] += mul * sum (beta^j * columns[j][i]) for j in 0..columns.len()`
/// where each column is padded with 0s as needed.
///
/// This is similar to adding `mul * fold_vals(beta,...)` on each row.

pub fn fold_columns(
    res: &mut [Curve25519Scalar],
    mul: Curve25519Scalar,
    beta: Curve25519Scalar,
    columns: &[impl MultilinearExtension<Curve25519Scalar>],
) {
    for (m, col) in powers(mul, beta).zip(columns) {
        col.mul_add(res, &m);
    }
}

/// This function takes a set of values and returns a scalar that is the
/// result of folding the values.
///
/// The result is
/// `sum (beta^j * vals[j]) for j in 0..vals.len()`
pub fn fold_vals(beta: Curve25519Scalar, vals: &[Curve25519Scalar]) -> Curve25519Scalar {
    let beta_powers = powers(Curve25519Scalar::one(), beta);
    beta_powers.zip(vals).map(|(pow, &val)| pow * val).sum()
}

/// Returns an iterator for the lazily evaluated sequence `init, init * base, init * base^2, ...`
fn powers(
    init: Curve25519Scalar,
    base: Curve25519Scalar,
) -> impl Iterator<Item = Curve25519Scalar> {
    core::iter::successors(Some(init), move |&m| Some(m * base))
}
