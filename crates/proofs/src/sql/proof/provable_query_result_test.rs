use super::{ProvableQueryResult, ProvableResultColumn};
use crate::{
    base::{
        database::{ColumnField, ColumnType},
        scalar::ArkScalar,
    },
    sql::proof::Indexes,
};
use arrow::{
    array::{Decimal128Array, Int64Array, StringArray},
    datatypes::{Field, Schema},
    record_batch::RecordBatch,
};
use num_traits::Zero;
use std::sync::Arc;

#[test]
fn we_can_convert_an_empty_provable_result_to_a_final_result() {
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new([0_i64; 0])];
    let res = ProvableQueryResult::new(&Indexes::Sparse(vec![]), &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)];
    let res = RecordBatch::try_from(res.into_owned_table(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(|v| v.into()).collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_evaluate_result_columns_as_mles() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i64; 3] = [10, 11, -12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];

    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    #[allow(clippy::possible_missing_comma)]
    let expected_evals =
        [ArkScalar::from(10u64) * evaluation_vec[0] - ArkScalar::from(12u64) * evaluation_vec[2]];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_result_columns_with_no_rows() {
    let indexes = Indexes::Sparse(vec![]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    let expected_evals = [ArkScalar::zero()];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values1: [i64; 3] = [10, 11, 12];
    let values2: [i64; 3] = [5, 7, 9];
    let cols: [Box<dyn ProvableResultColumn>; 2] = [Box::new(values1), Box::new(values2)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    let expected_evals = [
        ArkScalar::from(10u64) * evaluation_vec[0] + ArkScalar::from(12u64) * evaluation_vec[2],
        ArkScalar::from(5u64) * evaluation_vec[0] + ArkScalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_128_bits() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values1: [i128; 3] = [10, 11, 12];
    let values2: [i128; 3] = [5, 7, 9];
    let cols: [Box<dyn ProvableResultColumn>; 2] = [Box::new(values1), Box::new(values2)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::Int128); cols.len()];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    let expected_evals = [
        ArkScalar::from(10u64) * evaluation_vec[0] + ArkScalar::from(12u64) * evaluation_vec[2],
        ArkScalar::from(5u64) * evaluation_vec[0] + ArkScalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_scalar_columns() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values1: [ArkScalar; 3] = [10.into(), 11.into(), 12.into()];
    let values2: [ArkScalar; 3] = [5.into(), 7.into(), 9.into()];
    let cols: [Box<dyn ProvableResultColumn>; 2] = [Box::new(values1), Box::new(values2)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::Scalar); cols.len()];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    let expected_evals = [
        ArkScalar::from(10u64) * evaluation_vec[0] + ArkScalar::from(12u64) * evaluation_vec[2],
        ArkScalar::from(5u64) * evaluation_vec[0] + ArkScalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_mixed_data_types() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values1: [i64; 3] = [10, 11, 12];
    let values2: [i128; 3] = [5, 7, 9];
    let cols: [Box<dyn ProvableResultColumn>; 2] = [Box::new(values1), Box::new(values2)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields = [
        ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("a".parse().unwrap(), ColumnType::Int128),
    ];
    let evals = res.evaluate(&evaluation_vec, &column_fields[..]).unwrap();
    let expected_evals = [
        ArkScalar::from(10u64) * evaluation_vec[0] + ArkScalar::from(12u64) * evaluation_vec[2],
        ArkScalar::from(5u64) * evaluation_vec[0] + ArkScalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn evaluation_fails_if_indexes_are_out_of_range() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    match res.indexes_mut() {
        Indexes::Sparse(indexes) => indexes[1] = 20,
        _ => panic!("unexpected indexes type"),
    }
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(res.evaluate(&evaluation_vec, &column_fields[..]).is_none());
}

#[test]
fn evaluation_fails_if_indexes_are_not_sorted() {
    let indexes = Indexes::Sparse(vec![1, 0]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(res.evaluate(&evaluation_vec, &column_fields[..]).is_none());
}

#[test]
fn evaluation_fails_if_extra_data_is_included() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    res.data_mut().push(3u8);
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(res.evaluate(&evaluation_vec, &column_fields[..]).is_none());
}

#[test]
fn evaluation_fails_if_the_result_cant_be_decoded() {
    let mut res = ProvableQueryResult::new_from_raw_data(
        1,
        Indexes::Sparse(vec![0]),
        vec![0b11111111_u8; 38],
    );
    res.data_mut()[37] = 0b00000001_u8;
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); res.num_columns()];
    assert!(res.evaluate(&evaluation_vec, &column_fields[..]).is_none());
}

#[test]
fn evaluation_fails_if_data_is_missing() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    *res.num_columns_mut() = 3;
    let evaluation_vec = [
        ArkScalar::from(10u64),
        ArkScalar::from(100u64),
        ArkScalar::from(1000u64),
        ArkScalar::from(10000u64),
    ];
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); res.num_columns()];
    assert!(res.evaluate(&evaluation_vec, &column_fields[..]).is_none());
}

#[test]
fn we_can_convert_a_provable_result_to_a_final_result() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i64; 3] = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)];
    let res = RecordBatch::try_from(res.into_owned_table(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(|v| v.into()).collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![10, 12]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_128_bits() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values: [i128; 3] = [10, 11, i128::MAX];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::Int128)];
    let res = RecordBatch::try_from(res.into_owned_table(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(|v| v.into()).collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res = RecordBatch::try_new(
        schema,
        vec![Arc::new(
            Decimal128Array::from(vec![10, i128::MAX])
                .with_precision_and_scale(38, 0)
                .unwrap(),
        )],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_mixed_data_types() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values1: [i64; 3] = [6, 7, i64::MAX];
    let values2: [i128; 3] = [10, 11, i128::MAX];
    let values3 = ["abc".as_bytes(), &[0xed, 0xa0, 0x80][..], "de".as_bytes()];
    let cols: [Box<dyn ProvableResultColumn>; 3] =
        [Box::new(values1), Box::new(values2), Box::new(values3)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![
        ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("a2".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("a3".parse().unwrap(), ColumnType::VarChar),
    ];
    let res = RecordBatch::try_from(res.into_owned_table(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(|v| v.into()).collect();
    let schema = Arc::new(Schema::new(column_fields));
    println!("{:?}", res);
    let expected_res = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![6, i64::MAX])),
            Arc::new(
                Decimal128Array::from(vec![10, i128::MAX])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["abc", "de"])),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_cannot_convert_a_provable_result_with_invalid_string_data() {
    let values = ["abc".as_bytes(), &[0xed, 0xa0, 0x80][..], "de".as_bytes()];
    let cols: [Box<dyn ProvableResultColumn>; 1] = [Box::new(values)];
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::VarChar)];
    let indexes = Indexes::Sparse(vec![0]);
    assert!(ProvableQueryResult::new(&indexes, &cols)
        .into_owned_table(&column_fields)
        .is_ok());
    let indexes = Indexes::Sparse(vec![2]);
    assert!(ProvableQueryResult::new(&indexes, &cols)
        .into_owned_table(&column_fields)
        .is_ok());
    let indexes = Indexes::Sparse(vec![1]);
    assert!(ProvableQueryResult::new(&indexes, &cols)
        .into_owned_table(&column_fields)
        .is_err());
}

// TODO: we don't correctly detect overflow yet
// #[test]
// #[should_panic]
// fn we_can_detect_overflow() {
//     let indexes = [0];
//     let values = [i64::MAX];
//     let cols : [Box<dyn ProvableResultColumn>; 1] = [
//             Box::new(values),
//     ];
//     let res = ProvableQueryResult::new(&
//         &indexes,
//         &cols,
//     );
//     let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)];
//     let res = res.into_query_result(&column_fields).unwrap();
// }
