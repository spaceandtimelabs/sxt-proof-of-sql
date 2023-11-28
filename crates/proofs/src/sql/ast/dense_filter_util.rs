use crate::base::database::Column;
use bumpalo::Bump;

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
