use super::{
    Column, ColumnField, ColumnOperationError, ColumnOperationResult, ColumnType, Table,
    TableOperationError, TableOperationResult, TableOptions,
};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;

/// Check if two schemas are compatible
/// Note that we can tolerate differences in column names but not in column types
fn are_schemas_compatible(left: &[ColumnField], right: &[ColumnField]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(field1, field2)| field1.data_type() == field2.data_type())
}

/// Union multiple columns of the same type into a single column
///
/// # Panics
/// This function should never panic as long as it is written correctly
pub fn column_union<'a, S: Scalar>(
    columns: &[&Column<'a, S>],
    alloc: &'a Bump,
    column_type: ColumnType,
) -> ColumnOperationResult<Column<'a, S>> {
    // Check for type mismatch
    let possible_bad_column_type = columns.iter().find_map(|col| {
        let found_column_type = col.column_type();
        (found_column_type != column_type).then_some(found_column_type)
    });
    if let Some(bad_column_type) = possible_bad_column_type {
        return Err(ColumnOperationError::UnionDifferentTypes {
            actual_type: bad_column_type,
            correct_type: column_type,
        });
    }
    Ok(match column_type {
        ColumnType::Boolean => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_boolean().expect("Column types should match"))
                .copied()
                .collect();
            Column::Boolean(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::TinyInt => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_tinyint().expect("Column types should match"))
                .copied()
                .collect();
            Column::TinyInt(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::SmallInt => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_smallint().expect("Column types should match"))
                .copied()
                .collect();
            Column::SmallInt(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::Int => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_int().expect("Column types should match"))
                .copied()
                .collect();
            Column::Int(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::BigInt => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_bigint().expect("Column types should match"))
                .copied()
                .collect();
            Column::BigInt(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::Int128 => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_int128().expect("Column types should match"))
                .copied()
                .collect();
            Column::Int128(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::Scalar => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_scalar().expect("Column types should match"))
                .copied()
                .collect();
            Column::Scalar(alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::Decimal75(precision, scale) => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_decimal75().expect("Column types should match"))
                .copied()
                .collect();
            Column::Decimal75(precision, scale, alloc.alloc_slice_copy(&result) as &[_])
        }
        ColumnType::VarChar => {
            let (nested_result, nested_scalars): (Vec<_>, Vec<_>) = columns
                .iter()
                .map(|col| col.as_varchar().expect("Column types should match"))
                .unzip();
            let result: Vec<_> = nested_result.into_iter().flatten().copied().collect();
            let scalars: Vec<_> = nested_scalars.into_iter().flatten().copied().collect();
            Column::VarChar((
                alloc.alloc_slice_copy(&result) as &[_],
                alloc.alloc_slice_copy(&scalars) as &[_],
            ))
        }
        ColumnType::TimestampTZ(tu, tz) => {
            let result: Vec<_> = columns
                .iter()
                .flat_map(|col| col.as_timestamptz().expect("Column types should match"))
                .copied()
                .collect();
            Column::TimestampTZ(tu, tz, alloc.alloc_slice_copy(&result) as &[_])
        }
    })
}

/// Union multiple tables with compatible schemas into a single table
///
/// # Panics
/// This function should never panic as long as it is written correctly
pub fn table_union<'a, S: Scalar>(
    tables: &[&Table<'a, S>],
    alloc: &'a Bump,
    schema: Vec<ColumnField>,
) -> TableOperationResult<Table<'a, S>> {
    // Check schema equality
    let possible_bad_schema = tables
        .iter()
        .filter(|&table| (!are_schemas_compatible(&schema, &table.schema())))
        .map(|table| table.schema().clone())
        .next();
    if let Some(bad_schema) = possible_bad_schema {
        return Err(TableOperationError::UnionIncompatibleSchemas {
            actual_schema: bad_schema.clone(),
            correct_schema: schema,
        });
    }
    // Union the columns
    // Make sure to consider the case where the tables have no columns
    let num_rows = tables.iter().map(|table| table.num_rows()).sum();
    let result = Table::<'a, S>::try_from_iter_with_options(
        schema.iter().enumerate().map(|(i, field)| {
            let columns: Vec<_> = tables
                .iter()
                .map(|table| table.column(i).expect("Schemas should be compatible"))
                .collect();
            (
                field.name(),
                column_union(&columns, alloc, field.data_type()).expect("Failed to union columns"),
            )
        }),
        TableOptions::new(Some(num_rows)),
    )
    .expect("Failed to create table from iterator");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{map::IndexMap, scalar::test_scalar::TestScalar};

    #[test]
    fn we_can_union_no_columns() {
        let alloc = Bump::new();
        let result = column_union::<TestScalar>(&[], &alloc, ColumnType::BigInt).unwrap();
        assert_eq!(result, Column::BigInt(&[]));
    }

    #[test]
    fn we_can_union_columns_of_the_same_type() {
        let alloc = Bump::new();
        let col0: Column<TestScalar> = Column::BigInt(&[]);
        let col1: Column<TestScalar> = Column::BigInt(&[1, 2, 3]);
        let col2: Column<TestScalar> = Column::BigInt(&[4, 5, 6]);
        let col3: Column<TestScalar> = Column::BigInt(&[7, 8, 9]);
        let result =
            column_union(&[&col0, &col1, &col2, &col3], &alloc, ColumnType::BigInt).unwrap();
        assert_eq!(result, Column::BigInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9]));

        let strings = vec!["a", "b", "c"];
        let scalars = strings
            .iter()
            .map(|s| TestScalar::from(*s))
            .collect::<Vec<_>>();
        let col0: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let col1: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let result = column_union(&[&col0, &col1], &alloc, ColumnType::VarChar).unwrap();
        let doubled_strings: Vec<_> = strings.iter().chain(strings.iter()).copied().collect();
        let doubled_scalars: Vec<_> = scalars.iter().chain(scalars.iter()).copied().collect();
        assert_eq!(
            result,
            Column::VarChar((&doubled_strings, &doubled_scalars))
        );
    }

    #[test]
    fn we_cannot_union_columns_with_wrong_types() {
        let alloc = Bump::new();
        let col0: Column<TestScalar> = Column::BigInt(&[]);
        let result = column_union(&[&col0], &alloc, ColumnType::Int);
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnionDifferentTypes { .. })
        ));
    }

    #[test]
    fn we_can_union_no_tables() {
        let alloc = Bump::new();
        let result = table_union::<TestScalar>(&[], &alloc, vec![]).unwrap();
        assert_eq!(
            result,
            Table::<'_, TestScalar>::try_new_with_options(
                IndexMap::default(),
                TableOptions::new(Some(0))
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_union_tables_without_columns() {
        let alloc = Bump::new();
        let table0 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions::new(Some(2)),
        )
        .unwrap();
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions::new(Some(5)),
        )
        .unwrap();
        let table2 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions::new(Some(0)),
        )
        .unwrap();
        let result = table_union(&[&table0, &table1, &table2], &alloc, vec![]).unwrap();
        assert_eq!(
            result,
            Table::<'_, TestScalar>::try_new_with_options(
                IndexMap::default(),
                TableOptions::new(Some(7))
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_union_tables() {
        let alloc = Bump::new();
        // Column names don't matter
        let table0 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("a".parse().unwrap(), Column::BigInt(&[1, 2, 3])),
                ("b".parse().unwrap(), Column::BigInt(&[4, 5, 6])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("c".parse().unwrap(), Column::BigInt(&[7, 8, 9])),
                ("d".parse().unwrap(), Column::BigInt(&[10, 11, 12])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let result = table_union(
            &[&table0, &table1],
            &alloc,
            vec![
                ColumnField::new("e".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("f".parse().unwrap(), ColumnType::BigInt),
            ],
        )
        .unwrap();
        assert_eq!(
            result,
            Table::<'_, TestScalar>::try_new_with_options(
                IndexMap::from_iter(vec![
                    ("e".parse().unwrap(), Column::BigInt(&[1, 2, 3, 7, 8, 9])),
                    ("f".parse().unwrap(), Column::BigInt(&[4, 5, 6, 10, 11, 12])),
                ]),
                TableOptions::new(Some(6)),
            )
            .unwrap()
        );
    }

    #[test]
    fn we_cannot_union_tables_with_incompatible_schema() {
        let alloc = Bump::new();
        // Any difference in column types between a table and the result schema will do
        // regardless of whether the tables have the same schema
        let table0 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("a".parse().unwrap(), Column::BigInt(&[1, 2, 3])),
                ("b".parse().unwrap(), Column::BigInt(&[4, 5, 6])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("c".parse().unwrap(), Column::BigInt(&[7, 8, 9])),
                ("d".parse().unwrap(), Column::BigInt(&[10, 11, 12])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let result = table_union(
            &[&table0, &table1],
            &alloc,
            vec![
                ColumnField::new("e".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("f".parse().unwrap(), ColumnType::Int),
            ],
        );
        assert!(matches!(
            result,
            Err(TableOperationError::UnionIncompatibleSchemas { .. })
        ));
    }
}
