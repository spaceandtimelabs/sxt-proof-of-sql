use super::{OwnedColumn, OwnedTable};
use crate::{
    base::{database::OwnedArrowConversionError, scalar::ArkScalar},
    owned_table, record_batch,
};
use arrow::{
    array::{ArrayRef, Decimal128Array, Float32Array, Int64Array, StringArray},
    datatypes::Schema,
    record_batch::RecordBatch,
};
use indexmap::IndexMap;
use std::sync::Arc;

fn we_can_convert_between_owned_column_and_array_ref_impl(
    owned_column: OwnedColumn<ArkScalar>,
    array_ref: ArrayRef,
) {
    let ic_to_ar = ArrayRef::from(owned_column.clone());
    let ar_to_ic = OwnedColumn::try_from(array_ref.clone()).unwrap();

    assert!(ic_to_ar == array_ref);
    assert_eq!(owned_column, ar_to_ic);
}
fn we_can_convert_between_bigint_owned_column_and_array_ref_impl(data: Vec<i64>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<ArkScalar>::BigInt(data.clone()),
        Arc::new(Int64Array::from(data)),
    );
}
fn we_can_convert_between_int128_owned_column_and_array_ref_impl(data: Vec<i128>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<ArkScalar>::Int128(data.clone()),
        Arc::new(
            Decimal128Array::from(data)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        ),
    );
}
fn we_can_convert_between_varchar_owned_column_and_array_ref_impl(data: Vec<String>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        OwnedColumn::<ArkScalar>::VarChar(data.clone()),
        Arc::new(StringArray::from(data)),
    );
}
#[test]
fn we_can_convert_between_owned_column_and_array_ref() {
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_int128_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(vec![]);
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
        OwnedColumn::<ArkScalar>::try_from(array_ref),
        Err(OwnedArrowConversionError::UnsupportedType(_))
    ));
}

fn we_can_convert_between_owned_table_and_record_batch_impl(
    owned_table: OwnedTable<ArkScalar>,
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
        OwnedTable::<ArkScalar>::try_new(IndexMap::new()).unwrap(),
        RecordBatch::new_empty(Arc::new(Schema::empty())),
    );
    we_can_convert_between_owned_table_and_record_batch_impl(
        owned_table!(
            "a" => [0_i64; 0],
            "b" => [0_i128; 0],
            "c" => ["0"; 0],
        ),
        record_batch!(
            "a" => [0_i64; 0],
            "b" => [0_i128; 0],
            "c" => ["0"; 0],
        ),
    );
    we_can_convert_between_owned_table_and_record_batch_impl(
        owned_table!(
            "a" => [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            "b" => [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
            "c" => ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
        ),
        record_batch!(
            "a" => [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            "b" => [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
            "c" => ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
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
        OwnedTable::<ArkScalar>::try_from(record_batch),
        Err(OwnedArrowConversionError::DuplicateIdentifiers)
    ));
}

#[test]
#[should_panic]
fn we_panic_when_converting_an_owned_table_with_a_scalar_column() {
    let owned_table = owned_table!(
        "a" => [ArkScalar::from(0_i64); 0],
    );
    let _ = RecordBatch::try_from(owned_table);
}
