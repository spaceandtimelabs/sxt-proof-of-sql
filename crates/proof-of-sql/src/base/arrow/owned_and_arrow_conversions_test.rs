use super::owned_and_arrow_conversions::OwnedArrowConversionError;
use crate::base::{
    database::{owned_table_utility::*, OwnedColumn, OwnedNullableColumn, OwnedTable},
    map::IndexMap,
    scalar::test_scalar::TestScalar,
};
use alloc::sync::Arc;
use arrow::{
    array::{
        ArrayRef, BinaryArray, BooleanArray, Decimal128Array, Float32Array, Int64Array, StringArray,
    },
    datatypes::{DataType, Field, Schema},
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

proptest! {
    #[test]
    fn we_can_roundtrip_arbitrary_owned_column(owned_column: OwnedColumn<TestScalar>) {
        let arrow = ArrayRef::from(owned_column.clone());
        let actual = OwnedColumn::try_from(arrow).unwrap();

        prop_assert_eq!(actual, owned_column);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use arrow::array::{Array, ArrayRef, BooleanArray, Int32Array};

    #[test]
    fn we_can_convert_owned_nullable_column_to_array_ref_no_nulls() {
        let owned_column = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::new(owned_column);
        let array_ref: ArrayRef = nullable_column.into();
        let boolean_array = array_ref.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert_eq!(boolean_array.len(), 3);
        assert_eq!(boolean_array.null_count(), 0);
        assert!(boolean_array.value(0));
        assert!(!boolean_array.value(1));
        assert!(boolean_array.value(2));
    }

    #[test]
    fn we_can_convert_owned_nullable_column_to_array_ref_with_nulls() {
        let owned_column = OwnedColumn::<TestScalar>::Int(vec![10, 20, 30]);
        let presence = Some(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence);
        let array_ref: ArrayRef = nullable_column.into();
        let int_array = array_ref.as_any().downcast_ref::<Int32Array>().unwrap();
        assert_eq!(int_array.len(), 3);
        assert_eq!(int_array.null_count(), 1);
        assert_eq!(int_array.value(0), 10);
        assert!(int_array.is_null(1));
        assert_eq!(int_array.value(2), 30);
    }

    #[test]
    fn we_can_convert_array_ref_to_owned_nullable_column_no_nulls() {
        let array = Int32Array::from(vec![10, 20, 30]);
        let array_ref = Arc::new(array) as ArrayRef;
        let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(&array_ref).unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_nullable());

        match nullable_column.values {
            OwnedColumn::Int(values) => {
                assert_eq!(values, vec![10, 20, 30]);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn we_can_convert_array_ref_to_owned_nullable_column_with_nulls() {
        let array = Int32Array::from(vec![Some(10), None, Some(30)]);
        let array_ref = Arc::new(array) as ArrayRef;
        let nullable_column = OwnedNullableColumn::<TestScalar>::try_from(&array_ref).unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());
        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        match &nullable_column.values {
            OwnedColumn::Int(values) => {
                assert_eq!(values[0], 10);
                assert_eq!(values[1], 0);
                assert_eq!(values[2], 30);
            }
            _ => panic!("Expected Int column"),
        }
    }

    #[test]
    fn we_can_round_trip_nullable_column_through_arrow() {
        let owned_column = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
        let presence = Some(vec![true, false, true, false]);
        let original = OwnedNullableColumn::with_presence(owned_column, presence);
        let array_ref: ArrayRef = original.clone().into();
        let round_tripped = OwnedNullableColumn::<TestScalar>::try_from(&array_ref).unwrap();

        assert_eq!(round_tripped.len(), original.len());
        assert_eq!(round_tripped.is_nullable(), original.is_nullable());

        for i in 0..original.len() {
            assert_eq!(round_tripped.is_null(i), original.is_null(i));
        }

        match (&round_tripped.values, &original.values) {
            (OwnedColumn::Boolean(rt_values), OwnedColumn::Boolean(orig_values)) => {
                for i in 0..original.len() {
                    if !original.is_null(i) {
                        assert_eq!(rt_values[i], orig_values[i]);
                    }
                }
            }
            _ => panic!("Expected Boolean columns"),
        }
    }
}
