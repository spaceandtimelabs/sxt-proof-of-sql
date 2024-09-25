use super::{OwnedColumn, OwnedTable};
use crate::{
    base::{
        database::{owned_table_utility::*, OwnedArrowConversionError},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    record_batch,
};
use arrow::{
    array::{ArrayRef, BooleanArray, Decimal128Array, Float32Array, Int64Array, StringArray},
    datatypes::Schema,
    record_batch::RecordBatch,
};
use std::sync::Arc;

fn we_can_convert_between_owned_column_and_array_ref_impl(
    owned_column: OwnedColumn<TestScalar>,
    array_ref: ArrayRef,
) {
    let ic_to_ar = ArrayRef::from(owned_column.clone());
    let ar_to_ic = OwnedColumn::try_from(array_ref.clone()).unwrap();

    assert!(ic_to_ar == array_ref);
    assert_eq!(owned_column, ar_to_ic);
}
fn we_can_convert_between_boolean_owned_column_and_array_ref_impl(data: Vec<bool>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<TestScalar>::Boolean(data.clone()),
        Arc::new(BooleanArray::from(data)),
    );
}
fn we_can_convert_between_bigint_owned_column_and_array_ref_impl(data: Vec<i64>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<TestScalar>::BigInt(data.clone()),
        Arc::new(Int64Array::from(data)),
    );
}
fn we_can_convert_between_int128_owned_column_and_array_ref_impl(data: Vec<i128>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<TestScalar>::Int128(data.clone()),
        Arc::new(
            Decimal128Array::from(data)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        ),
    );
}
fn we_can_convert_between_varchar_owned_column_and_array_ref_impl(data: Vec<String>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<TestScalar>::VarChar(data.clone()),
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
}

#[test]
fn we_get_an_unsupported_type_error_when_trying_to_convert_from_a_float32_array_ref_to_an_owned_column(
) {
    let array_ref: ArrayRef = Arc::new(Float32Array::from(vec![0.0]));
    assert!(matches!(
        OwnedColumn::<TestScalar>::try_from(array_ref),
        Err(OwnedArrowConversionError::UnsupportedType(_))
    ));
}

fn we_can_convert_between_owned_table_and_record_batch_impl(
    owned_table: OwnedTable<TestScalar>,
    record_batch: RecordBatch,
) {
    let it_to_rb = RecordBatch::try_from(owned_table.clone()).unwrap();
    let rb_to_it = OwnedTable::try_from(record_batch.clone()).unwrap();

    assert_eq!(it_to_rb, record_batch);
    assert_eq!(rb_to_it, owned_table);
}
#[test]
fn we_can_convert_between_owned_table_and_record_batch() {
    we_can_convert_between_owned_table_and_record_batch_impl(
        OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap(),
        RecordBatch::new_empty(Arc::new(Schema::empty())),
    );
    we_can_convert_between_owned_table_and_record_batch_impl(
        owned_table([
            bigint("int64", [0; 0]),
            int128("int128", [0; 0]),
            varchar("string", ["0"; 0]),
            boolean("boolean", [true; 0]),
        ]),
        record_batch!(
            "int64" => [0_i64; 0],
            "int128" => [0_i128; 0],
            "string" => ["0"; 0],
            "boolean" => [true; 0],
        ),
    );
    we_can_convert_between_owned_table_and_record_batch_impl(
        owned_table([
            bigint("int64", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            int128("int128", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
            varchar("string", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
            boolean(
                "boolean",
                [true, false, true, false, true, false, true, false, true],
            ),
        ]),
        record_batch!(
            "int64" => [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            "int128" => [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
            "string" => ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
            "boolean" => [true, false, true, false, true, false, true, false, true],
        ),
    );
}

#[test]
fn we_cannot_convert_a_record_batch_if_it_has_repeated_column_names() {
    let record_batch = record_batch!(
        "a" => [0_i64; 0],
        "A" => [0_i128; 0],
    );
    assert!(matches!(
        OwnedTable::<TestScalar>::try_from(record_batch),
        Err(OwnedArrowConversionError::DuplicateIdentifiers)
    ));
}

#[test]
#[should_panic]
fn we_panic_when_converting_an_owned_table_with_a_scalar_column() {
    let owned_table = owned_table::<TestScalar>([scalar("a", [0; 0])]);
    let _ = RecordBatch::try_from(owned_table);
}
