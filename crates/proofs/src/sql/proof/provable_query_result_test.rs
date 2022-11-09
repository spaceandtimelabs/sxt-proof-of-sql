use super::{DenseProvableResultColumn, ProvableQueryResult, ProvableResultColumn};

use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use std::sync::Arc;

#[test]
fn we_can_convert_an_empty_provable_result_to_a_final_result() {
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::<i64>::new(&[][..]))];
    let res = ProvableQueryResult::new(&[][..], &cols);
    let schema = Schema::new(vec![Field::new("1", DataType::Int64, false)]);
    let schema = Arc::new(schema);
    let res = res.into_query_result(schema.clone()).unwrap();
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_evaluate_result_columns_as_mles() {
    let indexes = [0, 2];
    let values = [10, 11, -12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    let evals = res.evaluate(&evaluation_vec).unwrap();
    let expected_evals =
        [Scalar::from(10u64) * evaluation_vec[0] - Scalar::from(12u64) * evaluation_vec[2]];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_result_columns_with_no_rows() {
    let indexes = [];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    let evals = res.evaluate(&evaluation_vec).unwrap();
    let expected_evals = [Scalar::zero()];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles() {
    let indexes = [0, 2];
    let values1 = [10, 11, 12];
    let values2 = [5, 7, 9];
    let cols: [Box<dyn ProvableResultColumn>; 2] = [
        Box::new(DenseProvableResultColumn::new(&values1)),
        Box::new(DenseProvableResultColumn::new(&values2)),
    ];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    let evals = res.evaluate(&evaluation_vec).unwrap();
    let expected_evals = [
        Scalar::from(10u64) * evaluation_vec[0] + Scalar::from(12u64) * evaluation_vec[2],
        Scalar::from(5u64) * evaluation_vec[0] + Scalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn evaluation_fails_if_indexes_are_out_of_range() {
    let indexes = [0, 2];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    res.indexes[1] = 20;
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    assert!(res.evaluate(&evaluation_vec).is_none());
}

#[test]
fn evaluation_fails_if_indexes_are_not_sorted() {
    let indexes = [1, 0];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    assert!(res.evaluate(&evaluation_vec).is_none());
}

#[test]
fn evaluation_fails_if_extra_data_is_included() {
    let indexes = [0, 2];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    res.data.push(3u8);
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    assert!(res.evaluate(&evaluation_vec).is_none());
}

#[test]
fn evaluation_fails_if_the_result_cant_be_decoded() {
    let mut res = ProvableQueryResult {
        num_columns: 1,
        indexes: vec![0],
        data: vec![0b11111111_u8; 38],
    };
    res.data[37] = 0b00000001_u8;
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    assert!(res.evaluate(&evaluation_vec).is_none());
}

#[test]
fn evaluation_fails_if_data_is_missing() {
    let indexes = [0, 2];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    res.num_columns = 3;
    let evaluation_vec = [
        Scalar::from(10u64),
        Scalar::from(100u64),
        Scalar::from(1000u64),
        Scalar::from(10000u64),
    ];
    assert!(res.evaluate(&evaluation_vec).is_none());
}

#[test]
fn we_can_convert_an_provable_result_to_a_final_result() {
    let indexes = [0, 2];
    let values = [10, 11, 12];
    let cols: [Box<dyn ProvableResultColumn>; 1] =
        [Box::new(DenseProvableResultColumn::new(&values))];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let schema = Schema::new(vec![Field::new("1", DataType::Int64, false)]);
    let schema = Arc::new(schema);
    let res = res.into_query_result(schema.clone()).unwrap();
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![10, 12]))]).unwrap();
    assert_eq!(res, expected_res);
}

// TODO: we don't correctly detect overflow yet
// #[test]
// #[should_panic]
// fn we_can_detect_overflow() {
//     let indexes = [0];
//     let values = [i64::MAX];
//     let cols : [Box<dyn ProvableResultColumn>; 1] = [
//             Box::new(DenseProvableResultColumn::new(&values)),
//     ];
//     let res = ProvableQueryResult::new(
//         &indexes,
//         &cols,
//     );
//     let schema = Schema::new(vec![
//         Field::new("1", DataType::Int32, false),
//     ]);
//     let res = res.into_query_result(Arc::new(schema)).unwrap();
// }
