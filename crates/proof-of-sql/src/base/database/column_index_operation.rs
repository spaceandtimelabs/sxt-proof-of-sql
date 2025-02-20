use super::{Column, ColumnOperationResult, ColumnType, slice_operation::apply_slice_to_indexes};
use crate::base::scalar::Scalar;
use bumpalo::Bump;

/// Apply a `Column` to a vector of indexes, returning a new `Column` with the
/// values at the given indexes. Repetitions are allowed.
///
/// # Panics
/// Panics if any of the indexes are out of bounds.
#[allow(dead_code)]
pub(crate) fn apply_column_to_indexes<'a, S>(
    column: &Column<'a, S>,
    alloc: &'a Bump,
    indexes: &[usize],
) -> ColumnOperationResult<Column<'a, S>>
where
    S: Scalar,
{
    match column.column_type() {
        ColumnType::Boolean => {
            let raw_values = apply_slice_to_indexes(
                column.as_boolean().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Boolean(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::TinyInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_tinyint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::TinyInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Uint8 => {
            let raw_values = apply_slice_to_indexes(
                column.as_uint8().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Uint8(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::SmallInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_smallint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::SmallInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Int => {
            let raw_values = apply_slice_to_indexes(
                column.as_int().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Int(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::BigInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_bigint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::BigInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Int128 => {
            let raw_values = apply_slice_to_indexes(
                column.as_int128().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Int128(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Scalar => {
            let raw_values = apply_slice_to_indexes(
                column.as_scalar().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Scalar(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Decimal75(precision, scale) => {
            let raw_values = apply_slice_to_indexes(
                column.as_decimal75().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Decimal75(
                precision,
                scale,
                alloc.alloc_slice_copy(&raw_values) as &[_],
            ))
        }
        ColumnType::VarChar => {
            let (raw_values, raw_scalars) = column.as_varchar().expect("Column types should match");
            let raw_values = apply_slice_to_indexes(raw_values, indexes)?;
            let scalars = apply_slice_to_indexes(raw_scalars, indexes)?;
            Ok(Column::VarChar((
                alloc.alloc_slice_clone(&raw_values) as &[_],
                alloc.alloc_slice_copy(&scalars) as &[_],
            )))
        }

        ColumnType::VarBinary => {
            let (raw_values, raw_scalars) =
                column.as_varbinary().expect("Column types should match");
            let raw_values = apply_slice_to_indexes(raw_values, indexes)?;
            let scalars = apply_slice_to_indexes(raw_scalars, indexes)?;
            Ok(Column::VarBinary((
                alloc.alloc_slice_clone(&raw_values) as &[_],
                alloc.alloc_slice_copy(&scalars) as &[_],
            )))
        }
        ColumnType::TimestampTZ(tu, tz) => {
            let raw_values = apply_slice_to_indexes(
                column.as_timestamptz().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::TimestampTZ(
                tu,
                tz,
                alloc.alloc_slice_copy(&raw_values) as &[_],
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::ColumnOperationError, scalar::test_scalar::TestScalar};

    #[test]
    fn test_apply_index_op() {
        let bump = Bump::new();
        let column: Column<TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
        let indexes = [1, 3, 1, 2];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::Int(&[2, 4, 2, 3]));

        let scalars = [1, 2, 3].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::Scalar(&scalars);
        let indexes = [1, 1, 1];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_scalars = [2, 2, 2].iter().map(TestScalar::from).collect::<Vec<_>>();
        assert_eq!(result, Column::Scalar(&expected_scalars));

        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let indexes = [2, 1, 1];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_strings = vec!["c", "b", "b"];
        let expected_scalars = expected_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&expected_strings, &expected_scalars))
        );
    }

    #[test]
    fn test_apply_index_op_out_of_bound() {
        let bump = Bump::new();
        let column: Column<TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
        let indexes = [1, 3, 1, 2, 5];
        let result = apply_column_to_indexes(&column, &bump, &indexes);
        assert!(matches!(
            result,
            Err(ColumnOperationError::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_apply_index_op_varbinary() {
        let bump = Bump::new();

        let raw_bytes: Vec<&[u8]> = vec![b"foo".as_ref(), b"bar".as_ref(), b"baz".as_ref()];
        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let column = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));

        let indexes = [2, 0];

        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_bytes = vec![b"baz".as_ref(), b"foo".as_ref()];
        let expected_scalars: Vec<TestScalar> = expected_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        let expected = Column::VarBinary((expected_bytes.as_slice(), expected_scalars.as_slice()));

        assert_eq!(result, expected);
    }
}
