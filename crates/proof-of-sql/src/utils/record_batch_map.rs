use std::fmt::Display;

use arrow::array::{ArrayRef, RecordBatch};
use indexmap::IndexMap;
use snafu::Snafu;
use sqlparser::ast::DataType;

/// Common expect message for collecting into a record batch.
const EXPECT_TRY_FROM_ITER: &str =
    "Previously valid record batch should still satisfy all try_from_iter guarantees after mapping";

/// Returns the provided record batch with `f` applied to every column.
pub fn record_batch_map<F>(batch: RecordBatch, mut f: F) -> RecordBatch
where
    F: FnMut(ArrayRef) -> ArrayRef,
{
    RecordBatch::try_from_iter(
        batch
            .schema()
            .fields
            .into_iter()
            .zip(batch.columns().to_owned())
            .map(|(field, column)| (field.name(), f(column))),
    )
    .expect(EXPECT_TRY_FROM_ITER)
}

/// Could not find target type for a column.
#[derive(Debug, Snafu)]
#[snafu(display("could not find target type for {column_name}"))]
pub struct TargetTypeNotFound {
    /// The column without a target type.
    column_name: String,
}

/// Returns the provided record batch with a target-type-aware `f` applied to every column.
///
/// Errors if a column does not have a target type in the provided map.
pub fn record_batch_map_with_target_types<F>(
    batch: RecordBatch,
    target_types: &IndexMap<String, DataType>,
    mut f: F,
) -> Result<RecordBatch, TargetTypeNotFound>
where
    F: FnMut(ArrayRef, &DataType) -> ArrayRef,
{
    Ok(RecordBatch::try_from_iter(
        batch
            .schema()
            .fields
            .into_iter()
            .zip(batch.columns().to_owned())
            .map(|(field, column)| {
                let target_type =
                    target_types
                        .get(field.name())
                        .ok_or_else(|| TargetTypeNotFound {
                            column_name: field.name().clone(),
                        })?;

                Ok((field.name(), f(column, target_type)))
            })
            .collect::<Result<Vec<_>, TargetTypeNotFound>>()?,
    )
    .expect(EXPECT_TRY_FROM_ITER))
}

/// Errors that can occur when applying a fallible, target-type-aware map to a column.
#[derive(Debug, Snafu)]
pub enum MapOrTargetTypeError<E>
where
    E: Display,
{
    /// Unable to apply map to column.
    #[snafu(display("unable to apply map to column: {error}"))]
    MapFailure {
        /// The source error.
        error: E,
    },
    /// Could not find target type for a column.
    #[snafu(transparent)]
    TargetType {
        /// The source error.
        source: TargetTypeNotFound,
    },
}

impl<E> MapOrTargetTypeError<E>
where
    E: Display,
{
    /// Construct [`MapOrTargetTypeError::MapFailure`].
    fn map_failure(error: E) -> Self {
        MapOrTargetTypeError::MapFailure { error }
    }
}

/// Returns the provided record batch with a fallible, target-type-aware `f` applied to every
/// column.
///
/// Errors if a column does not have a target type in the provided map, or if `f` fails.
pub fn record_batch_try_map_with_target_types<F, E>(
    batch: RecordBatch,
    target_types: &IndexMap<String, DataType>,
    mut f: F,
) -> Result<RecordBatch, MapOrTargetTypeError<E>>
where
    F: FnMut(ArrayRef, &DataType) -> Result<ArrayRef, E>,
    E: Display,
{
    Ok(RecordBatch::try_from_iter(
        batch
            .schema()
            .fields
            .into_iter()
            .zip(batch.columns().to_owned())
            .map(|(field, column)| {
                let target_type =
                    target_types
                        .get(field.name())
                        .ok_or_else(|| TargetTypeNotFound {
                            column_name: field.name().clone(),
                        })?;

                Ok((
                    field.name(),
                    f(column, target_type).map_err(MapOrTargetTypeError::map_failure)?,
                ))
            })
            .collect::<Result<Vec<_>, MapOrTargetTypeError<E>>>()?,
    )
    .expect(EXPECT_TRY_FROM_ITER))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::array::{Decimal256Array, Int32Array, StringArray};
    use arrow::datatypes::i256;
    use sqlparser::ast::ExactNumberInfo;

    use super::*;

    // #[test]
    // fn we_can_map_record_batch() {
    //     let int_id = "int_col";
    //     let int_column: ArrayRef = Arc::new(Int32Array::from_iter([1, 4, -1, 0]));

    //     let varchar_id = "varchar_col";
    //     let varchar_column: ArrayRef = Arc::new(StringArray::from_iter([
    //         Some("lorem"),
    //         Some("i\0ps\0um"),
    //         None,
    //         Some("\0"),
    //     ]));
    //     let varchar_column_expected: ArrayRef = Arc::new(StringArray::from_iter([
    //         Some("lorem"),
    //         Some("ipsum"),
    //         None,
    //         Some(""),
    //     ]));

    //     let record_batch = RecordBatch::try_from_iter([
    //         (int_id, int_column.clone()),
    //         (varchar_id, varchar_column),
    //     ])
    //     .unwrap();

    //     let expected = RecordBatch::try_from_iter([
    //         (int_id, int_column),
    //         (varchar_id, varchar_column_expected),
    //     ])
    //     .unwrap();

    //     assert_eq!(
    //         record_batch_map(record_batch, column_remove_null_bytes),
    //         expected
    //     );
    // }

    // #[test]
    // fn we_can_map_record_batch_with_target_type() {
    //     let int_id = "int_col";
    //     let int_column: ArrayRef = Arc::new(Int32Array::from_iter([1, 4, -1]));

    //     let decimal_as_string_id = "decimal_col";
    //     let decimal_as_string_column: ArrayRef =
    //         Arc::new(StringArray::from_iter_values(["0", "-10.5", "2e4"]));
    //     let expected_decimal_column: ArrayRef = Arc::new(
    //         Decimal256Array::from_iter_values([
    //             i256::from_i128(0),
    //             i256::from_i128(-1050),
    //             i256::from_i128(2000000),
    //         ])
    //         .with_precision_and_scale(10, 2)
    //         .unwrap(),
    //     );

    //     let target_types = IndexMap::from_iter([
    //         (int_id.to_string(), DataType::Int(None)),
    //         (
    //             decimal_as_string_id.to_string(),
    //             DataType::Decimal(ExactNumberInfo::PrecisionAndScale(10, 2)),
    //         ),
    //     ]);

    //     let record_batch = RecordBatch::try_from_iter([
    //         (int_id, int_column.clone()),
    //         (decimal_as_string_id, decimal_as_string_column),
    //     ])
    //     .unwrap();

    //     let expected = RecordBatch::try_from_iter([
    //         (int_id, int_column),
    //         (decimal_as_string_id, expected_decimal_column),
    //     ])
    //     .unwrap();

    //     assert_eq!(
    //         record_batch_map_with_target_types(
    //             record_batch.clone(),
    //             &target_types,
    //             column_parse_decimals_unchecked
    //         )
    //         .unwrap(),
    //         expected.clone()
    //     );
    //     assert_eq!(
    //         record_batch_try_map_with_target_types(
    //             record_batch,
    //             &target_types,
    //             column_parse_decimals_fallible
    //         )
    //         .unwrap(),
    //         expected
    //     );
    // }

    // #[test]
    // fn we_cannot_map_record_batch_with_missing_target_type() {
    //     let int_id = "int_col";
    //     let int_column: ArrayRef = Arc::new(Int32Array::from_iter([1, 4, -1]));

    //     let decimal_as_string_id = "decimal_col";
    //     let decimal_as_string_column: ArrayRef =
    //         Arc::new(StringArray::from_iter_values(["0", "-10.5", "2e4"]));

    //     let target_types = IndexMap::from_iter([(
    //         decimal_as_string_id.to_string(),
    //         DataType::Decimal(ExactNumberInfo::PrecisionAndScale(10, 2)),
    //     )]);

    //     let record_batch = RecordBatch::try_from_iter([
    //         (int_id, int_column.clone()),
    //         (decimal_as_string_id, decimal_as_string_column),
    //     ])
    //     .unwrap();

    //     assert!(record_batch_map_with_target_types(
    //         record_batch.clone(),
    //         &target_types,
    //         column_parse_decimals_unchecked
    //     )
    //     .is_err());
    //     assert!(matches!(
    //         record_batch_try_map_with_target_types(
    //             record_batch,
    //             &target_types,
    //             column_parse_decimals_fallible
    //         ),
    //         Err(MapOrTargetTypeError::TargetType { .. })
    //     ));
    // }

    // #[test]
    // fn we_cannot_map_record_batch_with_map_failure() {
    //     let int_id = "int_col";
    //     let int_column: ArrayRef = Arc::new(Int32Array::from_iter([1, 4, -1]));

    //     let decimal_as_string_id = "decimal_col";
    //     let decimal_as_string_column: ArrayRef =
    //         Arc::new(StringArray::from_iter_values(["0", "not a decimal", "200"]));

    //     let target_types = IndexMap::from_iter([
    //         (int_id.to_string(), DataType::Int(None)),
    //         (
    //             decimal_as_string_id.to_string(),
    //             DataType::Decimal(ExactNumberInfo::PrecisionAndScale(10, 2)),
    //         ),
    //     ]);

    //     let record_batch = RecordBatch::try_from_iter([
    //         (int_id, int_column.clone()),
    //         (decimal_as_string_id, decimal_as_string_column),
    //     ])
    //     .unwrap();

    //     assert!(matches!(
    //         record_batch_try_map_with_target_types(
    //             record_batch,
    //             &target_types,
    //             column_parse_decimals_fallible
    //         ),
    //         Err(MapOrTargetTypeError::MapFailure { .. })
    //     ));
    // }
}
