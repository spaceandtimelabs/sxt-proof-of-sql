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
#[allow(clippy::too_many_lines)]
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
    // First, calculate the total length of the combined columns
    let len: usize = columns.iter().map(|col| col.len()).sum();

    Ok(match column_type {
        ColumnType::Boolean => {
            // Define a mutable iterator outside the closure
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_boolean().expect("Column types should match"))
                .copied();

            Column::Boolean(alloc.alloc_slice_fill_with(len, |_| {
                // Use iter.next() to get the next element
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::Uint8 => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_uint8().expect("Column types should match"))
                .copied();

            Column::Uint8(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::TinyInt => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_tinyint().expect("Column types should match"))
                .copied();

            Column::TinyInt(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::SmallInt => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_smallint().expect("Column types should match"))
                .copied();

            Column::SmallInt(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::Int => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_int().expect("Column types should match"))
                .copied();

            Column::Int(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::BigInt => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_bigint().expect("Column types should match"))
                .copied();

            Column::BigInt(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::Int128 => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_int128().expect("Column types should match"))
                .copied();

            Column::Int128(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::Scalar => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_scalar().expect("Column types should match"))
                .copied();

            Column::Scalar(alloc.alloc_slice_fill_with(len, |_| {
                iter.next().expect("Iterator should have enough elements")
            }) as &[_])
        }
        ColumnType::Decimal75(precision, scale) => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_decimal75().expect("Column types should match"))
                .copied();

            Column::Decimal75(
                precision,
                scale,
                alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_],
            )
        }
        ColumnType::VarChar => {
            let (nested_results, nested_scalars): (Vec<_>, Vec<_>) = columns
                .iter()
                .map(|col| col.as_varchar().expect("Column types should match"))
                .unzip();

            // Create iterators for both results and scalars
            let mut result_iter = nested_results.into_iter().flatten().copied();
            let mut scalar_iter = nested_scalars.into_iter().flatten().copied();

            Column::VarChar((
                alloc.alloc_slice_fill_with(len, |_| {
                    result_iter
                        .next()
                        .expect("Iterator should have enough elements")
                }) as &[_],
                alloc.alloc_slice_fill_with(len, |_| {
                    scalar_iter
                        .next()
                        .expect("Iterator should have enough elements")
                }) as &[_],
            ))
        }
        ColumnType::VarBinary => {
            let (nested_results, nested_scalars): (Vec<_>, Vec<_>) = columns
                .iter()
                .map(|col| col.as_varbinary().expect("Column types should match"))
                .unzip();

            // Create iterators for both results and scalars
            let mut result_iter = nested_results.into_iter().flatten().copied();
            let mut scalar_iter = nested_scalars.into_iter().flatten().copied();

            Column::VarBinary((
                alloc.alloc_slice_fill_with(len, |_| {
                    result_iter
                        .next()
                        .expect("Iterator should have enough elements")
                }) as &[_],
                alloc.alloc_slice_fill_with(len, |_| {
                    scalar_iter
                        .next()
                        .expect("Iterator should have enough elements")
                }) as &[_],
            ))
        }
        ColumnType::TimestampTZ(tu, tz) => {
            let mut iter = columns
                .iter()
                .flat_map(|col| col.as_timestamptz().expect("Column types should match"))
                .copied();

            Column::TimestampTZ(
                tu,
                tz,
                alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_],
            )
        }
        ColumnType::FixedSizeBinary(width) => {
            let mut iter = columns.iter().flat_map(|col| {
                let (col_width, col_data) = col
                    .as_fixed_size_binary()
                    .expect("Column should have the same type");
                assert_eq!(col_width, width, "Inconsistent FixedSizeBinary width");
                col_data.iter().copied()
            });

            Column::FixedSizeBinary(width, {
                let chunk_size: usize = width.into();
                alloc.alloc_slice_fill_with(len * chunk_size, |_| {
                    iter.next().expect("Iterator should have enough elements")
                })
            })
        }
    })
}

/// Union multiple tables with compatible schemas into a single table
///
/// # Panics
/// This function should never panic as long as it is written correctly
pub fn table_union<'a, S: Scalar>(
    tables: &[Table<'a, S>],
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
    let num_rows = tables.iter().map(Table::num_rows).sum();
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
        let result = table_union(&[table0, table1, table2], &alloc, vec![]).unwrap();
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
                ("a".into(), Column::BigInt(&[1, 2, 3])),
                ("b".into(), Column::BigInt(&[4, 5, 6])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("c".into(), Column::BigInt(&[7, 8, 9])),
                ("d".into(), Column::BigInt(&[10, 11, 12])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let result = table_union(
            &[table0, table1],
            &alloc,
            vec![
                ColumnField::new("e".into(), ColumnType::BigInt),
                ColumnField::new("f".into(), ColumnType::BigInt),
            ],
        )
        .unwrap();
        assert_eq!(
            result,
            Table::<'_, TestScalar>::try_new_with_options(
                IndexMap::from_iter(vec![
                    ("e".into(), Column::BigInt(&[1, 2, 3, 7, 8, 9])),
                    ("f".into(), Column::BigInt(&[4, 5, 6, 10, 11, 12])),
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
                ("a".into(), Column::BigInt(&[1, 2, 3])),
                ("b".into(), Column::BigInt(&[4, 5, 6])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![
                ("c".into(), Column::BigInt(&[7, 8, 9])),
                ("d".into(), Column::BigInt(&[10, 11, 12])),
            ]),
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let result = table_union(
            &[table0, table1],
            &alloc,
            vec![
                ColumnField::new("e".into(), ColumnType::BigInt),
                ColumnField::new("f".into(), ColumnType::Int),
            ],
        );
        assert!(matches!(
            result,
            Err(TableOperationError::UnionIncompatibleSchemas { .. })
        ));
    }

    #[test]
    fn we_can_union_varbinary_columns() {
        let alloc = Bump::new();

        let raw0 = [b"foo".as_ref(), b"bar".as_ref()];
        let scalars0: Vec<TestScalar> = raw0
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        let col0: Column<TestScalar> = Column::VarBinary((raw0.as_slice(), scalars0.as_slice()));

        let raw1 = [b"baz".as_ref(), b"qux".as_ref()];
        let scalars1: Vec<TestScalar> = raw1
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        let col1: Column<TestScalar> = Column::VarBinary((raw1.as_slice(), scalars1.as_slice()));

        let result = column_union(&[&col0, &col1], &alloc, ColumnType::VarBinary).unwrap();

        let expected_raw = [
            b"foo".as_ref(),
            b"bar".as_ref(),
            b"baz".as_ref(),
            b"qux".as_ref(),
        ];
        let expected_scalars: Vec<TestScalar> = expected_raw
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        assert_eq!(
            result,
            Column::VarBinary((expected_raw.as_slice(), expected_scalars.as_slice()))
        );
    }

    #[test]
    fn we_can_union_tables_with_varbinary_columns() {
        let alloc = Bump::new();
        let binary_binding = [b"foo".as_ref(), b"bar".as_ref()];
        let scalar_binding = [
            TestScalar::from_le_bytes_mod_order(b"foo"),
            TestScalar::from_le_bytes_mod_order(b"bar"),
        ];
        let table0 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![(
                "vb".into(),
                Column::VarBinary((binary_binding.as_slice(), scalar_binding.as_slice())),
            )]),
            TableOptions::new(Some(2)),
        )
        .unwrap();

        let binary_binding = [b"baz".as_ref(), b"qux".as_ref()];
        let scalar_binding2 = [
            TestScalar::from_le_bytes_mod_order(b"baz"),
            TestScalar::from_le_bytes_mod_order(b"qux"),
        ];
        let table1 = Table::<'_, TestScalar>::try_new_with_options(
            IndexMap::from_iter(vec![(
                "some_name".into(),
                Column::VarBinary((binary_binding.as_slice(), scalar_binding2.as_slice())),
            )]),
            TableOptions::new(Some(2)),
        )
        .unwrap();

        let schema = vec![ColumnField::new(
            "doesnt_matter".into(),
            ColumnType::VarBinary,
        )];
        let result = table_union(&[table0, table1], &alloc, schema.clone()).unwrap();

        let expected_raw = [
            b"foo".as_ref(),
            b"bar".as_ref(),
            b"baz".as_ref(),
            b"qux".as_ref(),
        ];
        let expected_scalars: Vec<TestScalar> = expected_raw
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let expected = Table::try_new_with_options(
            IndexMap::from_iter(vec![(
                "doesnt_matter".into(),
                Column::VarBinary((expected_raw.as_slice(), expected_scalars.as_slice())),
            )]),
            TableOptions::new(Some(4)),
        )
        .unwrap();

        assert_eq!(result, expected);
    }
}
