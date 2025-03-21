use super::owned_and_arrow_conversions::OwnedArrowConversionError;
use crate::base::{
    database::{owned_table_utility::*, OwnedColumn, OwnedNullableColumn, OwnedTable, TableError},
    map::IndexMap,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use alloc::sync::Arc;
use arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, Decimal128Array, Decimal256Array, Float32Array,
        Int16Array, Int32Array, Int64Array, Int8Array, StringArray, UInt8Array,
    },
    datatypes::{i256, DataType, Field, Schema, TimeUnit as ArrowTimeUnit},
    record_batch::RecordBatch,
};
use proptest::prelude::*;

fn we_can_convert_between_owned_column_and_array_ref_impl(
    owned_column: &OwnedColumn<TestScalar>,
    array_ref: ArrayRef,
) {
    let ic_to_ar = ArrayRef::from(owned_column.clone());
    let ar_to_ic = OwnedColumn::try_from(array_ref.clone()).unwrap();

    assert!(ic_to_ar == array_ref);
    assert_eq!(*owned_column, ar_to_ic);
}

fn we_can_convert_between_varbinary_owned_column_and_array_ref_impl(data: &[Vec<u8>]) {
    let owned_col = OwnedColumn::<TestScalar>::VarBinary(data.to_owned());
    let arrow_col = Arc::new(BinaryArray::from(
        data.iter()
            .map(std::vec::Vec::as_slice)
            .collect::<Vec<&[u8]>>(),
    ));
    we_can_convert_between_owned_column_and_array_ref_impl(&owned_col, arrow_col);
}

fn we_can_convert_between_boolean_owned_column_and_array_ref_impl(data: Vec<bool>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Boolean(data.clone()),
        Arc::new(BooleanArray::from(data)),
    );
}
fn we_can_convert_between_bigint_owned_column_and_array_ref_impl(data: Vec<i64>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::BigInt(data.clone()),
        Arc::new(Int64Array::from(data)),
    );
}
fn we_can_convert_between_int128_owned_column_and_array_ref_impl(data: Vec<i128>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Int128(data.clone()),
        Arc::new(
            Decimal128Array::from(data)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        ),
    );
}
fn we_can_convert_between_varchar_owned_column_and_array_ref_impl(data: Vec<String>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::VarChar(data.clone()),
        Arc::new(StringArray::from(data)),
    );
}
#[test]
fn we_can_convert_between_owned_column_and_array_ref() {
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_int128_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(vec![]);
    let data = vec![true, false, true, false, true, false, true, false, true];
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX];
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX];
    we_can_convert_between_int128_owned_column_and_array_ref_impl(data);
    let data = vec!["0", "1", "2", "3", "4", "5", "6"];
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(
        data.into_iter().map(String::from).collect(),
    );

    let varbin_data = vec![
        b"foo".to_vec(),
        b"bar".to_vec(),
        b"baz".to_vec(),
        vec![],
        b"some bytes".to_vec(),
    ];
    we_can_convert_between_varbinary_owned_column_and_array_ref_impl(&varbin_data);
}

#[test]
fn we_get_an_unsupported_type_error_when_trying_to_convert_from_a_float32_array_ref_to_an_owned_column(
) {
    let array_ref: ArrayRef = Arc::new(Float32Array::from(vec![0.0]));
    assert!(matches!(
        OwnedColumn::<TestScalar>::try_from(array_ref),
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));
}

fn we_can_convert_between_owned_table_and_record_batch_impl(
    owned_table: &OwnedTable<TestScalar>,
    record_batch: &RecordBatch,
) {
    let it_to_rb = RecordBatch::try_from(owned_table.clone()).unwrap();
    let rb_to_it = OwnedTable::try_from(record_batch.clone()).unwrap();

    assert_eq!(it_to_rb, *record_batch);
    assert_eq!(rb_to_it, *owned_table);
}
#[test]
fn we_can_convert_between_owned_table_and_record_batch() {
    we_can_convert_between_owned_table_and_record_batch_impl(
        &OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap(),
        &RecordBatch::new_empty(Arc::new(Schema::empty())),
    );

    let schema = Arc::new(Schema::new(vec![
        Field::new("int64", DataType::Int64, false),
        Field::new("int128", DataType::Decimal128(38, 0), false),
        Field::new("string", DataType::Utf8, false),
        Field::new("boolean", DataType::Boolean, false),
    ]));

    let batch1 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![0_i64; 0])),
            Arc::new(
                Decimal128Array::from(vec![0_i128; 0])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["0"; 0])),
            Arc::new(BooleanArray::from(vec![true; 0])),
        ],
    )
    .unwrap();

    we_can_convert_between_owned_table_and_record_batch_impl(
        &owned_table([
            bigint("int64", [0; 0]),
            int128("int128", [0; 0]),
            varchar("string", ["0"; 0]),
            boolean("boolean", [true; 0]),
        ]),
        &batch1,
    );

    let batch2 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                i64::MIN,
                i64::MAX,
            ])),
            Arc::new(
                Decimal128Array::from(vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8",
            ])),
            Arc::new(BooleanArray::from(vec![
                true, false, true, false, true, false, true, false, true,
            ])),
        ],
    )
    .unwrap();

    we_can_convert_between_owned_table_and_record_batch_impl(
        &owned_table([
            bigint("int64", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            int128("int128", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
            varchar("string", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
            boolean(
                "boolean",
                [true, false, true, false, true, false, true, false, true],
            ),
        ]),
        &batch2,
    );
}

#[test]
#[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
fn we_panic_when_converting_an_owned_table_with_a_scalar_column() {
    let owned_table = owned_table::<TestScalar>([scalar("a", [0; 0])]);
    let _ = RecordBatch::try_from(owned_table);
}

#[test]
fn table_error_converts_to_owned_arrow_conversion_error() {
    let table_error = TableError::ColumnLengthMismatch;
    let conversion_error: OwnedArrowConversionError = table_error.into();
    if let OwnedArrowConversionError::TableError { source } = conversion_error {
        assert_eq!(source, TableError::ColumnLengthMismatch);
    } else {
        panic!("Wrong error variant for ColumnLengthMismatch");
    }

    let table_error = TableError::ColumnLengthMismatchWithSpecifiedRowCount;
    let conversion_error: OwnedArrowConversionError = table_error.into();
    if let OwnedArrowConversionError::TableError { source } = conversion_error {
        assert_eq!(
            source,
            TableError::ColumnLengthMismatchWithSpecifiedRowCount
        );
    } else {
        panic!("Wrong error variant for ColumnLengthMismatchWithSpecifiedRowCount");
    }

    let table_error = TableError::EmptyTableWithoutSpecifiedRowCount;
    let conversion_error: OwnedArrowConversionError = table_error.into();
    if let OwnedArrowConversionError::TableError { source } = conversion_error {
        assert_eq!(source, TableError::EmptyTableWithoutSpecifiedRowCount);
    } else {
        panic!("Wrong error variant for EmptyTableWithoutSpecifiedRowCount");
    }

    let table_error = TableError::PresenceLengthMismatch;
    let conversion_error: OwnedArrowConversionError = table_error.into();
    if let OwnedArrowConversionError::TableError { source } = conversion_error {
        assert_eq!(source, TableError::PresenceLengthMismatch);
    } else {
        panic!("Wrong error variant for PresenceLengthMismatch");
    }

    let table_error = TableError::ColumnNotFound {
        column: "test_column".to_string(),
    };
    let conversion_error: OwnedArrowConversionError = table_error.into();
    if let OwnedArrowConversionError::TableError { source } = conversion_error {
        if let TableError::ColumnNotFound { column } = source {
            assert_eq!(column, "test_column");
        } else {
            panic!("Wrong TableError variant for ColumnNotFound");
        }
    } else {
        panic!("Wrong error variant for ColumnNotFound");
    }
}

proptest! {
    #[test]
    fn we_can_roundtrip_arbitrary_owned_column(owned_column: OwnedColumn<TestScalar>) {
        let arrow = ArrayRef::from(owned_column.clone());
        let actual = OwnedColumn::try_from(arrow).unwrap();

        prop_assert_eq!(actual, owned_column);
    }
}

#[test]
fn we_can_convert_from_owned_nullable_column_to_array_ref_non_nullable() {
    let data = vec![true, false, true];
    let owned_col = OwnedColumn::<TestScalar>::Boolean(data.clone());
    let nullable_col = OwnedNullableColumn::new(owned_col);

    let array_ref = ArrayRef::from(nullable_col);

    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 0);
    let boolean_array = array_ref.as_any().downcast_ref::<BooleanArray>().unwrap();
    assert!(boolean_array.value(0));
    assert!(!boolean_array.value(1));
    assert!(boolean_array.value(2));
}

#[test]
#[allow(clippy::too_many_lines)]
fn we_can_convert_from_owned_nullable_column_to_array_ref_with_nulls() {
    let data = vec![true, false, true];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::Boolean(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let boolean_array = array_ref.as_any().downcast_ref::<BooleanArray>().unwrap();
    assert!(boolean_array.is_null(1));
    assert!(boolean_array.is_valid(0));
    assert!(boolean_array.is_valid(2));

    let data = vec![1u8, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::Uint8(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let uint8_array = array_ref.as_any().downcast_ref::<UInt8Array>().unwrap();
    assert!(uint8_array.is_null(1));
    assert_eq!(uint8_array.value(0), 1);
    assert_eq!(uint8_array.value(2), 3);

    let data = vec![1i8, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::TinyInt(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let int8_array = array_ref.as_any().downcast_ref::<Int8Array>().unwrap();
    assert!(int8_array.is_null(1));
    assert_eq!(int8_array.value(0), 1);
    assert_eq!(int8_array.value(2), 3);

    let data = vec![1i16, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::SmallInt(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let int16_array = array_ref.as_any().downcast_ref::<Int16Array>().unwrap();
    assert!(int16_array.is_null(1));
    assert_eq!(int16_array.value(0), 1);
    assert_eq!(int16_array.value(2), 3);

    let data = vec![1i32, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::Int(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let int32_array = array_ref.as_any().downcast_ref::<Int32Array>().unwrap();
    assert!(int32_array.is_null(1));
    assert_eq!(int32_array.value(0), 1);
    assert_eq!(int32_array.value(2), 3);

    let data = vec![1i64, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::BigInt(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let int64_array = array_ref.as_any().downcast_ref::<Int64Array>().unwrap();
    assert!(int64_array.is_null(1));
    assert_eq!(int64_array.value(0), 1);
    assert_eq!(int64_array.value(2), 3);

    let data = vec![1i128, 2, 3];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::Int128(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let decimal128_array = array_ref
        .as_any()
        .downcast_ref::<Decimal128Array>()
        .unwrap();
    assert!(decimal128_array.is_null(1));
    assert_eq!(decimal128_array.value(0), 1);
    assert_eq!(decimal128_array.value(2), 3);

    let data = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::VarChar(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let string_array = array_ref.as_any().downcast_ref::<StringArray>().unwrap();
    assert!(string_array.is_null(1));
    assert_eq!(string_array.value(0), "a");
    assert_eq!(string_array.value(2), "c");

    let data = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
    let presence = Some(vec![true, false, true]);
    let owned_col = OwnedColumn::<TestScalar>::VarBinary(data.clone());
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);
    let binary_array = array_ref.as_any().downcast_ref::<BinaryArray>().unwrap();
    assert!(binary_array.is_null(1));
    assert_eq!(binary_array.value(0), b"a");
    assert_eq!(binary_array.value(2), b"c");
}

#[test]
#[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
fn we_panic_when_converting_a_nullable_owned_column_with_scalar_values() {
    let data = vec![
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ];
    let owned_col = OwnedColumn::<TestScalar>::Scalar(data);
    let nullable_col = OwnedNullableColumn::new(owned_col);
    let _ = ArrayRef::from(nullable_col);
}

#[test]
fn we_can_convert_nullable_decimal75_owned_column_to_array_ref() {
    use crate::base::math::decimal::Precision;

    let precision = Precision::new(38).unwrap();
    let scale = 2;
    let data = vec![
        TestScalar::from(12345),
        TestScalar::from(67890),
        TestScalar::from(98765),
    ];
    let presence = Some(vec![true, false, true]);

    let owned_col = OwnedColumn::<TestScalar>::Decimal75(precision, scale, data);
    let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

    let array_ref = ArrayRef::from(nullable_col);
    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 1);

    let decimal256_array = array_ref
        .as_any()
        .downcast_ref::<Decimal256Array>()
        .unwrap();
    assert!(decimal256_array.is_null(1));
    assert_eq!(decimal256_array.precision(), 38);
    assert_eq!(decimal256_array.scale(), 2);
}

#[test]
fn we_can_convert_varchar_with_long_strings_to_array_ref() {
    let data = vec![
        "short".to_string(),
        "This is a medium length string that needs more capacity".to_string(),
        "This is an even longer string that should require a significant amount of capacity in the StringBuilder. We want to make sure that the capacity calculations in the StringBuilder conversion logic are tested properly.".to_string(),
        String::new(),
        "Another medium string to ensure we have enough data".to_string(),
    ];

    let presence_patterns = vec![
        Some(vec![true, true, true, true, true]),
        Some(vec![false, true, true, true, true]),
        Some(vec![true, false, true, true, true]),
        Some(vec![true, true, false, true, true]),
        Some(vec![true, true, true, false, true]),
        Some(vec![true, true, true, true, false]),
        Some(vec![true, false, true, false, true]),
        Some(vec![false, false, false, false, false]),
    ];

    for presence in presence_patterns {
        let owned_col = OwnedColumn::<TestScalar>::VarChar(data.clone());
        let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence.clone()).unwrap();

        let array_ref = ArrayRef::from(nullable_col);

        assert_eq!(array_ref.len(), data.len());

        let expected_null_count = if let Some(ref p) = presence {
            p.iter().filter(|&&present| !present).count()
        } else {
            0
        };
        assert_eq!(array_ref.null_count(), expected_null_count);

        let string_array = array_ref.as_any().downcast_ref::<StringArray>().unwrap();
        for i in 0..data.len() {
            if let Some(ref p) = presence {
                if p[i] {
                    assert!(!string_array.is_null(i));
                    assert_eq!(string_array.value(i), data[i]);
                } else {
                    assert!(string_array.is_null(i));
                }
            } else {
                assert!(!string_array.is_null(i));
                assert_eq!(string_array.value(i), data[i]);
            }
        }
    }
}

#[test]
fn we_can_convert_varbinary_with_varying_sizes_to_array_ref() {
    let data = vec![
        b"small".to_vec(),
        vec![0; 100],
        vec![1; 1000],
        vec![],
        b"another small one".to_vec(),
    ];

    let presence_patterns = vec![
        Some(vec![true, true, true, true, true]),
        Some(vec![false, true, true, true, true]),
        Some(vec![true, false, true, true, true]),
        Some(vec![true, true, false, true, true]),
        Some(vec![true, true, true, false, true]),
        Some(vec![true, true, true, true, false]),
        Some(vec![true, false, true, false, true]),
        Some(vec![false, false, false, false, false]),
    ];

    for presence in presence_patterns {
        let owned_col = OwnedColumn::<TestScalar>::VarBinary(data.clone());
        let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence.clone()).unwrap();

        let array_ref = ArrayRef::from(nullable_col);

        assert_eq!(array_ref.len(), data.len());

        let expected_null_count = if let Some(ref p) = presence {
            p.iter().filter(|&&present| !present).count()
        } else {
            0
        };
        assert_eq!(array_ref.null_count(), expected_null_count);

        let binary_array = array_ref.as_any().downcast_ref::<BinaryArray>().unwrap();
        for i in 0..data.len() {
            if let Some(ref p) = presence {
                if p[i] {
                    assert!(!binary_array.is_null(i));
                    assert_eq!(binary_array.value(i), data[i]);
                } else {
                    assert!(binary_array.is_null(i));
                }
            } else {
                assert!(!binary_array.is_null(i));
                assert_eq!(binary_array.value(i), data[i]);
            }
        }
    }
}

#[test]
fn we_can_convert_from_owned_nullable_timestamptz_column_to_array_ref() {
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use arrow::array::{
        TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
        TimestampSecondArray,
    };

    let time_units = vec![
        PoSQLTimeUnit::Second,
        PoSQLTimeUnit::Millisecond,
        PoSQLTimeUnit::Microsecond,
        PoSQLTimeUnit::Nanosecond,
    ];

    for time_unit in time_units {
        let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
        let presence = Some(vec![true, false, true]);
        let timezone = PoSQLTimeZone::utc();

        let owned_col = OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, data.clone());
        let nullable_col = OwnedNullableColumn::with_presence(owned_col, presence).unwrap();

        let array_ref = ArrayRef::from(nullable_col);

        assert_eq!(array_ref.len(), 3);
        assert_eq!(array_ref.null_count(), 1);

        match time_unit {
            PoSQLTimeUnit::Second => {
                let timestamp_array = array_ref
                    .as_any()
                    .downcast_ref::<TimestampSecondArray>()
                    .unwrap();
                assert!(timestamp_array.is_null(1));
                assert_eq!(timestamp_array.value(0), 1_625_072_400);
                assert_eq!(timestamp_array.value(2), 1_625_079_600);
            }
            PoSQLTimeUnit::Millisecond => {
                let timestamp_array = array_ref
                    .as_any()
                    .downcast_ref::<TimestampMillisecondArray>()
                    .unwrap();
                assert!(timestamp_array.is_null(1));
                assert_eq!(timestamp_array.value(0), 1_625_072_400);
                assert_eq!(timestamp_array.value(2), 1_625_079_600);
            }
            PoSQLTimeUnit::Microsecond => {
                let timestamp_array = array_ref
                    .as_any()
                    .downcast_ref::<TimestampMicrosecondArray>()
                    .unwrap();
                assert!(timestamp_array.is_null(1));
                assert_eq!(timestamp_array.value(0), 1_625_072_400);
                assert_eq!(timestamp_array.value(2), 1_625_079_600);
            }
            PoSQLTimeUnit::Nanosecond => {
                let timestamp_array = array_ref
                    .as_any()
                    .downcast_ref::<TimestampNanosecondArray>()
                    .unwrap();
                assert!(timestamp_array.is_null(1));
                assert_eq!(timestamp_array.value(0), 1_625_072_400);
                assert_eq!(timestamp_array.value(2), 1_625_079_600);
            }
        }
    }

    let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
    let timezone = PoSQLTimeZone::utc();

    let owned_col =
        OwnedColumn::<TestScalar>::TimestampTZ(PoSQLTimeUnit::Second, timezone, data.clone());
    let nullable_col = OwnedNullableColumn::new(owned_col);

    let array_ref = ArrayRef::from(nullable_col);

    assert_eq!(array_ref.len(), 3);
    assert_eq!(array_ref.null_count(), 0);
    let timestamp_array = array_ref
        .as_any()
        .downcast_ref::<TimestampSecondArray>()
        .unwrap();
    assert_eq!(timestamp_array.value(0), 1_625_072_400);
    assert_eq!(timestamp_array.value(1), 1_625_076_000);
    assert_eq!(timestamp_array.value(2), 1_625_079_600);
}

#[test]
fn we_can_convert_from_array_ref_to_owned_nullable_column() {
    let mut boolean_builder = arrow::array::BooleanBuilder::new();
    boolean_builder.append_value(true);
    boolean_builder.append_null();
    boolean_builder.append_value(false);
    let array_ref = Arc::new(boolean_builder.finish()) as ArrayRef;

    let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(array_ref.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::Boolean(values) = &nullable_column.values {
        assert_eq!(*values, vec![true, false, false]); // The middle value is arbitrary since it's null
        assert_eq!(nullable_column.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected Boolean column");
    }

    let array_ref_after = ArrayRef::from(nullable_column);

    assert_eq!(array_ref_after.len(), 3);
    assert_eq!(array_ref_after.null_count(), 1);
    let boolean_array = array_ref_after
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();
    assert!(boolean_array.is_valid(0));
    assert!(boolean_array.is_null(1));
    assert!(boolean_array.is_valid(2));
    assert!(boolean_array.value(0));
    assert!(!boolean_array.value(2));
}

#[test]
fn we_can_convert_from_uint8_array_to_owned_column() {
    let array_ref = Arc::new(UInt8Array::from(vec![10u8, 20, 30])) as ArrayRef;

    let owned_col = OwnedColumn::<TestScalar>::try_from(array_ref.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::Uint8(values) = owned_col {
        assert_eq!(values, vec![10, 20, 30]);
    } else {
        panic!("Expected Uint8 column");
    }

    let mut uint8_builder = arrow::array::UInt8Builder::new();
    uint8_builder.append_value(10);
    uint8_builder.append_null();
    uint8_builder.append_value(30);
    let array_ref_with_nulls = Arc::new(uint8_builder.finish()) as ArrayRef;

    let nullable_col =
        OwnedNullableColumn::<TestScalar>::try_from(array_ref_with_nulls.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::Uint8(_values) = &nullable_col.values {
        assert_eq!(nullable_col.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected Uint8 column");
    }
}

#[test]
fn we_can_convert_from_int8_array_to_owned_column() {
    let array_ref = Arc::new(Int8Array::from(vec![10i8, -20, 30])) as ArrayRef;

    let owned_col = OwnedColumn::<TestScalar>::try_from(array_ref.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::TinyInt(values) = owned_col {
        assert_eq!(values, vec![10, -20, 30]);
    } else {
        panic!("Expected TinyInt column");
    }

    let mut int8_builder = arrow::array::Int8Builder::new();
    int8_builder.append_value(10);
    int8_builder.append_null();
    int8_builder.append_value(30);
    let array_ref_with_nulls = Arc::new(int8_builder.finish()) as ArrayRef;

    let nullable_col =
        OwnedNullableColumn::<TestScalar>::try_from(array_ref_with_nulls.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::TinyInt(values) = &nullable_col.values {
        assert_eq!(*values, vec![10, -99, 30]);
        assert_eq!(nullable_col.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected TinyInt column");
    }
}

#[test]
fn we_can_directly_convert_from_int8_array_with_nulls_to_owned_column() {
    let mut int8_builder = arrow::array::Int8Builder::new();
    int8_builder.append_value(10);
    int8_builder.append_null();
    int8_builder.append_value(30);
    let array_ref = Arc::new(int8_builder.finish()) as ArrayRef;
    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Int8 => {
            let array = array_ref.as_any().downcast_ref::<Int8Array>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -99
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::TinyInt(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::TinyInt(values) = owned_column {
        assert_eq!(values, vec![10, -99, 30]);
    } else {
        panic!("Expected TinyInt column");
    }
}

#[test]
fn we_can_directly_convert_from_int16_array_with_nulls_to_owned_column() {
    let mut int16_builder = arrow::array::Int16Builder::new();
    int16_builder.append_value(10);
    int16_builder.append_null();
    int16_builder.append_value(30);
    let array_ref = Arc::new(int16_builder.finish()) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Int16 => {
            let array = array_ref.as_any().downcast_ref::<Int16Array>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -9999
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::SmallInt(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::SmallInt(values) = owned_column {
        assert_eq!(values, vec![10, -9999, 30]);
    } else {
        panic!("Expected SmallInt column");
    }
}

#[test]
fn we_can_directly_convert_from_int32_array_with_nulls_to_owned_column() {
    let mut int32_builder = arrow::array::Int32Builder::new();
    int32_builder.append_value(10);
    int32_builder.append_null();
    int32_builder.append_value(30);
    let array_ref = Arc::new(int32_builder.finish()) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Int32 => {
            let array = array_ref.as_any().downcast_ref::<Int32Array>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -999_999_999
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::Int(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::Int(values) = owned_column {
        assert_eq!(values, vec![10, -999_999_999, 30]);
    } else {
        panic!("Expected Int column");
    }
}

#[test]
fn we_can_directly_convert_from_int64_array_with_nulls_to_owned_column() {
    let mut int64_builder = arrow::array::Int64Builder::new();
    int64_builder.append_value(10);
    int64_builder.append_null();
    int64_builder.append_value(30);
    let array_ref = Arc::new(int64_builder.finish()) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Int64 => {
            let array = array_ref.as_any().downcast_ref::<Int64Array>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -999_999_999_999
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::BigInt(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::BigInt(values) = owned_column {
        assert_eq!(values, vec![10, -999_999_999_999, 30]);
    } else {
        panic!("Expected BigInt column");
    }
}

#[test]
fn we_can_directly_convert_from_decimal128_array_with_nulls_to_owned_column() {
    let mut decimal128_builder = arrow::array::Decimal128Builder::new();
    decimal128_builder.append_value(10);
    decimal128_builder.append_null();
    decimal128_builder.append_value(30);
    let array = decimal128_builder.finish();
    let array = array.with_precision_and_scale(38, 0).unwrap();
    let array_ref = Arc::new(array) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Decimal128(38, 0) => {
            let array = array_ref
                .as_any()
                .downcast_ref::<Decimal128Array>()
                .unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -999_999_999_999_999_999
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::Int128(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::Int128(values) = owned_column {
        assert_eq!(values, vec![10, -999_999_999_999_999_999, 30]);
    } else {
        panic!("Expected Int128 column");
    }
}

#[test]
fn test_int64_array_downcast_explicitly() {
    let data = vec![42i64, 123, 789];
    let int64_array = Int64Array::from(data.clone());
    let array_ref = Arc::new(int64_array) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref).unwrap();

    match result {
        OwnedColumn::<TestScalar>::BigInt(values) => {
            assert_eq!(values, data);
        }
        _ => panic!("Expected BigInt column"),
    }
}

#[test]
fn test_int64_array_conversion_with_nulls_explicitly() {
    let mut int64_builder = arrow::array::Int64Builder::new();
    int64_builder.append_value(42);
    int64_builder.append_null();
    int64_builder.append_value(123);
    let array_ref = Arc::new(int64_builder.finish()) as ArrayRef;

    let owned_column = match array_ref.data_type() {
        DataType::Int64 => {
            let array = array_ref.as_any().downcast_ref::<Int64Array>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -999_999_999_999
                } else {
                    array.value(i)
                });
            }
            OwnedColumn::<TestScalar>::BigInt(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::BigInt(values) = owned_column {
        assert_eq!(values, vec![42, -999_999_999_999, 123]);
    } else {
        panic!("Expected BigInt column");
    }
}

#[test]
fn test_decimal256_array_to_owned_nullable_column_with_nulls() {
    use arrow::array::Decimal256Builder;
    let mut decimal256_builder = Decimal256Builder::new();
    decimal256_builder.append_value(i256::from_i128(12345));
    decimal256_builder.append_null();
    decimal256_builder.append_value(i256::from_i128(67890));
    let array = decimal256_builder.finish();
    let array = array.with_precision_and_scale(38, 2).unwrap();
    let array_ref = Arc::new(array) as ArrayRef;

    let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(array_ref.clone()).unwrap();

    if let OwnedColumn::<TestScalar>::Decimal75(precision, scale, values) = &nullable_column.values
    {
        assert_eq!(precision.value(), 38);
        assert_eq!(*scale, 2);
        assert_eq!(values.len(), 3);
        assert_eq!(nullable_column.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected Decimal75 column");
    }

    let result = OwnedColumn::<TestScalar>::try_from(array_ref);
    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));
}

#[test]
fn test_decimal128_array_downcast_explicitly() {
    let data = vec![42i128, 123, 789];
    let decimal128_array = Decimal128Array::from(data.clone());
    let decimal128_array = decimal128_array.with_precision_and_scale(38, 0).unwrap();

    let array_ref = Arc::new(decimal128_array) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref).unwrap();

    match result {
        OwnedColumn::<TestScalar>::Int128(values) => {
            assert_eq!(values, data);
        }
        _ => panic!("Expected Int128 column"),
    }
}

#[test]
fn test_decimal256_array_with_nulls_to_owned_column() {
    let mut decimal256_builder = arrow::array::Decimal256Builder::new();
    decimal256_builder.append_value(i256::from_i128(12345));
    decimal256_builder.append_null();
    decimal256_builder.append_value(i256::from_i128(67890));
    let array = decimal256_builder.finish();
    let array = array.with_precision_and_scale(38, 2).unwrap();
    let array_ref = Arc::new(array) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Decimal256(38, 2) => {
            let array = array_ref
                .as_any()
                .downcast_ref::<Decimal256Array>()
                .unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    -999_999_999_999_999_999
                } else {
                    array.value(i).to_i128().unwrap()
                });
            }
            OwnedColumn::<TestScalar>::Int128(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::Int128(values) = owned_column {
        assert_eq!(values, vec![12345, -999_999_999_999_999_999, 67890]);
    } else {
        panic!("Expected Int128 column");
    }
}

#[test]
fn test_decimal256_array_to_owned_column_with_nulls_using_scalar_zero() {
    use crate::base::math::decimal::Precision;
    use arrow::{array::Decimal256Builder, datatypes::i256};

    let mut decimal256_builder = Decimal256Builder::new();
    decimal256_builder.append_value(i256::from_i128(12345));
    decimal256_builder.append_null();
    decimal256_builder.append_value(i256::from_i128(67890));
    let array = decimal256_builder.finish();
    let array = array.with_precision_and_scale(38, 2).unwrap();
    let array_ref = Arc::new(array) as ArrayRef;

    match OwnedColumn::<TestScalar>::try_from(array_ref.clone()) {
        Ok(_) => panic!("Expected conversion to fail due to nulls"),
        Err(err) => match err {
            OwnedArrowConversionError::UnsupportedType { .. } => {}
            _ => panic!("Unexpected error type: {err:?}"),
        },
    }

    let precision = 38;
    let scale = 2;
    let len = array_ref.len();

    let owned_column = match array_ref.data_type() {
        DataType::Decimal256(p, _s) if *p <= 75 => {
            let array = array_ref
                .as_any()
                .downcast_ref::<Decimal256Array>()
                .unwrap();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                let val = if array.is_null(i) {
                    TestScalar::from(0)
                } else {
                    TestScalar::from(array.value(i).to_i128().unwrap())
                };
                values.push(val);
            }

            let precision = Precision::new(precision).unwrap();
            OwnedColumn::<TestScalar>::Decimal75(precision, scale, values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::Decimal75(p, s, values) = owned_column {
        assert_eq!(p.value(), precision);
        assert_eq!(s, scale);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], TestScalar::from(12345));
        assert_eq!(values[1], TestScalar::from(0));
        assert_eq!(values[2], TestScalar::from(67890));
    } else {
        panic!("Expected Decimal75 column");
    }

    let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(array_ref).unwrap();

    if let OwnedColumn::<TestScalar>::Decimal75(p, s, values) = &nullable_column.values {
        assert_eq!(p.value(), precision);
        assert_eq!(*s, scale);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], TestScalar::from(12345));
        assert_eq!(values[1], TestScalar::from(0));
        assert_eq!(values[2], TestScalar::from(67890));
        assert_eq!(nullable_column.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected Decimal75 column in nullable column");
    }
}

#[test]
fn test_decimal256_array_conversion_fails_for_invalid_i256() {
    use arrow::{array::Decimal256Builder, datatypes::i256};

    let mut decimal256_builder = Decimal256Builder::new();

    let too_large_i256 = i256::from_parts(
        u128::MAX, // low bits - maximum value
        i128::MAX, // high bits - maximum positive value
    );

    decimal256_builder.append_value(too_large_i256);
    let array = decimal256_builder.finish();
    let array = array.with_precision_and_scale(38, 0).unwrap();
    let array_ref = Arc::new(array) as ArrayRef;

    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::DecimalConversionFailed { .. })
    ));

    let result = OwnedNullableColumn::<TestScalar>::try_from(array_ref);

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::DecimalConversionFailed { .. })
    ));
}

#[test]
fn we_can_directly_convert_from_string_array_with_nulls_to_owned_column() {
    let mut string_builder = arrow::array::StringBuilder::new();
    string_builder.append_value("hello");
    string_builder.append_null();
    string_builder.append_value("world");
    let array_ref = Arc::new(string_builder.finish()) as ArrayRef;
    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Utf8 => {
            let array = array_ref.as_any().downcast_ref::<StringArray>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    String::new()
                } else {
                    array.value(i).to_string()
                });
            }
            OwnedColumn::<TestScalar>::VarChar(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::VarChar(values) = owned_column {
        assert_eq!(
            values,
            vec!["hello".to_string(), String::new(), "world".to_string()]
        );
    } else {
        panic!("Expected VarChar column");
    }
}

#[test]
fn we_can_directly_convert_from_binary_array_with_nulls_to_owned_column() {
    let mut binary_builder = arrow::array::BinaryBuilder::new();
    binary_builder.append_value(b"hello");
    binary_builder.append_null();
    binary_builder.append_value(b"world");
    let array_ref = Arc::new(binary_builder.finish()) as ArrayRef;
    let result = OwnedColumn::<TestScalar>::try_from(array_ref.clone());

    assert!(matches!(
        result,
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));

    let owned_column = match array_ref.data_type() {
        DataType::Binary => {
            let array = array_ref.as_any().downcast_ref::<BinaryArray>().unwrap();
            let len = array_ref.len();
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                values.push(if array.is_null(i) {
                    Vec::new()
                } else {
                    array.value(i).to_vec()
                });
            }
            OwnedColumn::<TestScalar>::VarBinary(values)
        }
        _ => panic!("Unexpected data type"),
    };

    if let OwnedColumn::<TestScalar>::VarBinary(values) = owned_column {
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], b"hello");
        assert_eq!(values[1], Vec::<u8>::new());
        assert_eq!(values[2], b"world");
    } else {
        panic!("Expected VarBinary column");
    }

    let nullable_col = OwnedNullableColumn::<TestScalar>::try_from(array_ref).unwrap();

    if let OwnedColumn::<TestScalar>::VarBinary(values) = &nullable_col.values {
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], b"hello");
        assert_eq!(values[1], Vec::<u8>::new());
        assert_eq!(values[2], b"world");
        assert_eq!(nullable_col.presence, Some(vec![true, false, true]));
    } else {
        panic!("Expected VarBinary column in nullable column");
    }
}

#[test]
fn we_can_convert_from_timestamp_array_ref_to_owned_column() {
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use arrow::array::{
        TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
        TimestampSecondArray,
    };

    let time_units = vec![
        (
            PoSQLTimeUnit::Second,
            DataType::Timestamp(ArrowTimeUnit::Second, None),
        ),
        (
            PoSQLTimeUnit::Millisecond,
            DataType::Timestamp(ArrowTimeUnit::Millisecond, None),
        ),
        (
            PoSQLTimeUnit::Microsecond,
            DataType::Timestamp(ArrowTimeUnit::Microsecond, None),
        ),
        (
            PoSQLTimeUnit::Nanosecond,
            DataType::Timestamp(ArrowTimeUnit::Nanosecond, None),
        ),
    ];

    for (time_unit, _data_type) in time_units {
        let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
        let array_ref = match time_unit {
            PoSQLTimeUnit::Second => Arc::new(TimestampSecondArray::from(data.clone())) as ArrayRef,
            PoSQLTimeUnit::Millisecond => {
                Arc::new(TimestampMillisecondArray::from(data.clone())) as ArrayRef
            }
            PoSQLTimeUnit::Microsecond => {
                Arc::new(TimestampMicrosecondArray::from(data.clone())) as ArrayRef
            }
            PoSQLTimeUnit::Nanosecond => {
                Arc::new(TimestampNanosecondArray::from(data.clone())) as ArrayRef
            }
        };

        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_ref).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, values) => {
                assert_eq!(tu, time_unit);
                assert_eq!(tz, PoSQLTimeZone::utc());
                assert_eq!(values, data);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }
}

#[test]
fn we_can_convert_from_timestamp_array_with_timezone_to_owned_column() {
    use arrow::array::TimestampSecondArray;

    let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
    let array = TimestampSecondArray::from(data.clone()).with_timezone("+02:00");
    let array_ref = Arc::new(array) as ArrayRef;
    let owned_col = OwnedColumn::<TestScalar>::try_from(array_ref).unwrap();

    match owned_col {
        OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, values) => {
            assert_eq!(tu, PoSQLTimeUnit::Second);
            assert_eq!(tz, PoSQLTimeZone::new(7200));
            assert_eq!(values, data);
        }
        _ => panic!("Expected TimestampTZ column"),
    }
}

#[test]
fn we_can_convert_from_owned_column_to_timestamp_array_with_timezone() {
    use arrow::{array::TimestampSecondArray, datatypes::TimeUnit as ArrowTimeUnit};

    let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
    let timezone = PoSQLTimeZone::new(7200);
    let owned_col =
        OwnedColumn::<TestScalar>::TimestampTZ(PoSQLTimeUnit::Second, timezone, data.clone());

    let array_ref = ArrayRef::from(owned_col);

    assert_eq!(array_ref.len(), 3);
    match array_ref.data_type() {
        DataType::Timestamp(time_unit, Some(tz)) => {
            assert_eq!(*time_unit, ArrowTimeUnit::Second);
            assert_eq!(tz.as_ref(), "+02:00");
        }
        _ => panic!("Expected Timestamp type with timezone"),
    }

    let timestamp_array = array_ref
        .as_any()
        .downcast_ref::<TimestampSecondArray>()
        .unwrap();
    assert_eq!(timestamp_array.values(), &data[..]);
}

#[test]
fn we_can_convert_from_timestamp_millisecond_array_with_nulls_to_owned_nullable_column() {
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    let data = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();
    let mut builder = arrow::array::TimestampMillisecondBuilder::new();
    builder.append_value(data[0]);
    builder.append_value(data[1]);
    builder.append_null();

    let array = builder.finish();
    let array_ref = Arc::new(array) as ArrayRef;
    let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(array_ref).unwrap();

    if let OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, values) = &nullable_column.values {
        assert_eq!(*tu, PoSQLTimeUnit::Millisecond);
        assert_eq!(*tz, PoSQLTimeZone::utc());
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], data[0]);
        assert_eq!(values[1], data[1]);
        assert_eq!(values[2], -888_888_888_888);
        assert_eq!(nullable_column.presence, Some(vec![true, true, false]));
    } else {
        panic!("Expected TimestampTZ column");
    }

    let mut builder = arrow::array::TimestampMillisecondBuilder::new();
    builder.append_value(data[0]);
    builder.append_value(data[1]);
    builder.append_null();

    let array = builder.finish().with_timezone("+02:00");
    let array_ref = Arc::new(array) as ArrayRef;
    let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(array_ref).unwrap();

    if let OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, values) = &nullable_column.values {
        assert_eq!(*tu, PoSQLTimeUnit::Millisecond);
        assert_eq!(*tz, PoSQLTimeZone::new(7200));
        assert_eq!(values[0], data[0]);
        assert_eq!(values[1], data[1]);
        assert_eq!(values[2], -888_888_888_888);
        assert_eq!(nullable_column.presence, Some(vec![true, true, false]));
    } else {
        panic!("Expected TimestampTZ column");
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_timestamp_arrays_with_no_nulls_conversion_to_owned_column() {
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use arrow::array::{
        TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
        TimestampSecondArray,
    };

    let timestamps = [1_625_072_400, 1_625_076_000, 1_625_079_600].to_vec();

    {
        let array = Arc::new(TimestampSecondArray::from(timestamps.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Second);
                assert_eq!(timezone, PoSQLTimeZone::utc());
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }

        let array_with_tz =
            Arc::new(TimestampSecondArray::from(timestamps.clone()).with_timezone("+02:00"))
                as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_with_tz).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Second);
                assert_eq!(timezone, PoSQLTimeZone::new(7200));
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    {
        let array = Arc::new(TimestampMillisecondArray::from(timestamps.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Millisecond);
                assert_eq!(timezone, PoSQLTimeZone::utc());
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }

        let array_with_tz =
            Arc::new(TimestampMillisecondArray::from(timestamps.clone()).with_timezone("+02:00"))
                as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_with_tz).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Millisecond);
                assert_eq!(timezone, PoSQLTimeZone::new(7200));
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    {
        let array = Arc::new(TimestampMicrosecondArray::from(timestamps.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Microsecond);
                assert_eq!(timezone, PoSQLTimeZone::utc());
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }

        let array_with_tz =
            Arc::new(TimestampMicrosecondArray::from(timestamps.clone()).with_timezone("+02:00"))
                as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_with_tz).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Microsecond);
                assert_eq!(timezone, PoSQLTimeZone::new(7200));
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    {
        let array = Arc::new(TimestampNanosecondArray::from(timestamps.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Nanosecond);
                assert_eq!(timezone, PoSQLTimeZone::utc());
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }

        let array_with_tz =
            Arc::new(TimestampNanosecondArray::from(timestamps.clone()).with_timezone("+02:00"))
                as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_with_tz).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, values) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Nanosecond);
                assert_eq!(timezone, PoSQLTimeZone::new(7200));
                assert_eq!(values, timestamps);
            }
            _ => panic!("Expected TimestampTZ column"),
        }
    }

    {
        let array = Arc::new(Float32Array::from(vec![1.0, 2.0, 3.0])) as ArrayRef;
        let result = OwnedColumn::<TestScalar>::try_from(&array);
        assert!(matches!(
            result,
            Err(OwnedArrowConversionError::UnsupportedType { .. })
        ));
    }
}

#[test]
fn test_string_and_binary_arrays_conversion_to_owned_column() {
    {
        let string_data = vec!["first", "second", "third"];
        let array = Arc::new(StringArray::from(string_data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::VarChar(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], "first");
                assert_eq!(values[1], "second");
                assert_eq!(values[2], "third");
            }
            _ => panic!("Expected VarChar column"),
        }
    }

    {
        let binary_data: Vec<&[u8]> = vec![b"first", b"second", b"third"];
        let array = Arc::new(BinaryArray::from(binary_data)) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::VarBinary(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], b"first");
                assert_eq!(values[1], b"second");
                assert_eq!(values[2], b"third");
            }
            _ => panic!("Expected VarBinary column"),
        }
    }
}

#[test]
fn test_decimal256_array_conversion_to_owned_column() {
    use crate::base::math::decimal::Precision;
    use arrow::datatypes::i256;

    {
        let data = vec![
            i256::from_i128(100),
            i256::from_i128(200),
            i256::from_i128(300),
        ];

        let precision = 75;
        let scale = 2;
        let array = Decimal256Array::from(data.clone())
            .with_precision_and_scale(precision, scale)
            .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_ref).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::Decimal75(p, s, values) => {
                assert_eq!(p, Precision::new(precision).unwrap());
                assert_eq!(s, scale);
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], TestScalar::from(100));
                assert_eq!(values[1], TestScalar::from(200));
                assert_eq!(values[2], TestScalar::from(300));
            }
            _ => panic!("Expected Decimal75 column"),
        }
    }

    {
        let data = vec![i256::from_i128(100)];
        let array = Decimal256Array::from(data)
            .with_precision_and_scale(76, 2) // Precision > 75
            .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;
        let result = OwnedColumn::<TestScalar>::try_from(&array_ref);
        assert!(matches!(
            result,
            Err(OwnedArrowConversionError::UnsupportedType { .. })
        ));
    }

    {
        let large_value = i256::MAX;
        let data = vec![large_value];
        let array = Decimal256Array::from(data)
            .with_precision_and_scale(75, 0)
            .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;
        let result = OwnedColumn::<TestScalar>::try_from(&array_ref);
        assert!(matches!(
            result,
            Err(OwnedArrowConversionError::DecimalConversionFailed { .. })
        ));
    }
}

#[test]
fn test_integer_arrays_conversion_to_owned_column() {
    {
        let data = vec![1u8, 2u8, 3u8];
        let array = Arc::new(UInt8Array::from(data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::Uint8(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected Uint8 column"),
        }
    }

    {
        let data = vec![1i8, 2i8, 3i8];
        let array = Arc::new(Int8Array::from(data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::TinyInt(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected TinyInt column"),
        }
    }

    {
        let data = vec![1i16, 2i16, 3i16];
        let array = Arc::new(Int16Array::from(data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::SmallInt(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected SmallInt column"),
        }
    }

    {
        let data = vec![1i32, 2i32, 3i32];
        let array = Arc::new(Int32Array::from(data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::Int(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected Int column"),
        }
    }

    {
        let data = vec![1i64, 2i64, 3i64];
        let array = Arc::new(Int64Array::from(data.clone())) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::BigInt(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected BigInt column"),
        }
    }

    {
        let data = vec![1i128, 2i128, 3i128];
        let array = Decimal128Array::from(data.clone())
            .with_precision_and_scale(38, 0)
            .unwrap();
        let array_ref = Arc::new(array) as ArrayRef;
        let owned_col = OwnedColumn::<TestScalar>::try_from(&array_ref).unwrap();

        match owned_col {
            OwnedColumn::<TestScalar>::Int128(values) => {
                assert_eq!(values, data);
            }
            _ => panic!("Expected Int128 column"),
        }
    }
}

#[test]
fn test_boolean_array_conversion_to_owned_column() {
    let data = vec![true, false, true];
    let array = Arc::new(BooleanArray::from(data.clone())) as ArrayRef;
    let owned_col = OwnedColumn::<TestScalar>::try_from(&array).unwrap();

    match owned_col {
        OwnedColumn::<TestScalar>::Boolean(values) => {
            assert_eq!(values, data);
        }
        _ => panic!("Expected Boolean column"),
    }
}
