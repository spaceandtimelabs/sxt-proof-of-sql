use super::{DenseIntermediateResultColumn, IntermediateQueryResult, IntermediateResultColumn};

use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[test]
fn we_can_convert_an_empty_intermediate_result_to_a_final_result() {
    let cols: [Box<dyn IntermediateResultColumn>; 1] =
        [Box::new(DenseIntermediateResultColumn::<i64>::new(&[][..]))];
    let res = IntermediateQueryResult::new(&[][..], &cols);
    let schema = Schema::new(vec![Field::new("1", DataType::Int64, false)]);
    let schema = Arc::new(schema);
    let res = res.into_query_result(schema.clone()).unwrap();
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_convert_an_intermediate_result_to_a_final_result() {
    let indexes = [0, 2];
    let values = [10, 11, 12];
    let cols: [Box<dyn IntermediateResultColumn>; 1] =
        [Box::new(DenseIntermediateResultColumn::new(&values))];
    let res = IntermediateQueryResult::new(&indexes, &cols);
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
//     let cols : [Box<dyn IntermediateResultColumn>; 1] = [
//             Box::new(DenseIntermediateResultColumn::new(&values)),
//     ];
//     let res = IntermediateQueryResult::new(
//         &indexes,
//         &cols,
//     );
//     let schema = Schema::new(vec![
//         Field::new("1", DataType::Int32, false),
//     ]);
//     let res = res.into_query_result(Arc::new(schema)).unwrap();
// }
