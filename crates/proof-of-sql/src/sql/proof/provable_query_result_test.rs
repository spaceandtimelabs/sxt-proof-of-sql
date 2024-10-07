use super::{ProvableQueryResult, QueryError};
use crate::{
    base::{
        database::{Column, ColumnField, ColumnType},
        math::decimal::Precision,
        polynomial::compute_evaluation_vector,
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::proof::Indexes,
};
use alloc::sync::Arc;
use arrow::{
    array::{Decimal128Array, Decimal256Array, Int64Array, StringArray},
    datatypes::{i256, Field, Schema},
    record_batch::RecordBatch,
};
use num_traits::Zero;

#[test]
fn we_can_convert_an_empty_provable_result_to_a_final_result() {
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[0_i64; 0])];
    let res = ProvableQueryResult::new(&Indexes::Sparse(vec![]), &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(std::convert::Into::into).collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_evaluate_result_columns_as_mles() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, -12])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);

    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    #[allow(clippy::possible_missing_comma)]
    let expected_evals = [Curve25519Scalar::from(10u64) * evaluation_vec[0]
        - Curve25519Scalar::from(12u64) * evaluation_vec[2]];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_result_columns_with_no_rows() {
    let indexes = Indexes::Sparse(vec![]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    let expected_evals = [Curve25519Scalar::zero()];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 2] =
        [Column::BigInt(&[10, 11, 12]), Column::BigInt(&[5, 7, 9])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    let expected_evals = [
        Curve25519Scalar::from(10u64) * evaluation_vec[0]
            + Curve25519Scalar::from(12u64) * evaluation_vec[2],
        Curve25519Scalar::from(5u64) * evaluation_vec[0]
            + Curve25519Scalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_128_bits() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 2] =
        [Column::Int128(&[10, 11, 12]), Column::Int128(&[5, 7, 9])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::Int128); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    let expected_evals = [
        Curve25519Scalar::from(10u64) * evaluation_vec[0]
            + Curve25519Scalar::from(12u64) * evaluation_vec[2],
        Curve25519Scalar::from(5u64) * evaluation_vec[0]
            + Curve25519Scalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[allow(clippy::similar_names)]
#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_scalar_columns() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let col0 = [10, 11, 12]
        .iter()
        .map(|v| Curve25519Scalar::from(*v))
        .collect::<Vec<_>>();
    let col1 = [5, 7, 9]
        .iter()
        .map(|v| Curve25519Scalar::from(*v))
        .collect::<Vec<_>>();
    let cols: [Column<Curve25519Scalar>; 2] = [Column::Scalar(&col0), Column::Scalar(&col1)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::Scalar); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    let expected_evals = [
        Curve25519Scalar::from(10u64) * evaluation_vec[0]
            + Curve25519Scalar::from(12u64) * evaluation_vec[2],
        Curve25519Scalar::from(5u64) * evaluation_vec[0]
            + Curve25519Scalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_mixed_data_types() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 2] =
        [Column::BigInt(&[10, 11, 12]), Column::Int128(&[5, 7, 9])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = [
        ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("a".parse().unwrap(), ColumnType::Int128),
    ];
    let evals = res
        .evaluate(&evaluation_point, 4, &column_fields[..])
        .unwrap();
    let expected_evals = [
        Curve25519Scalar::from(10u64) * evaluation_vec[0]
            + Curve25519Scalar::from(12u64) * evaluation_vec[2],
        Curve25519Scalar::from(5u64) * evaluation_vec[0]
            + Curve25519Scalar::from(9u64) * evaluation_vec[2],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn evaluation_fails_if_indexes_are_out_of_range() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    match res.indexes_mut() {
        Indexes::Sparse(indexes) => indexes[1] = 20,
        _ => panic!("unexpected indexes type"),
    }
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::InvalidIndexes)
    ));
}

#[test]
fn evaluation_fails_if_indexes_are_not_sorted() {
    let indexes = Indexes::Sparse(vec![1, 0]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::InvalidIndexes)
    ));
}

#[test]
fn evaluation_fails_if_extra_data_is_included() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    res.data_mut().push(3u8);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); cols.len()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::MiscellaneousEvaluationError)
    ));
}

#[test]
fn evaluation_fails_if_the_result_cant_be_decoded() {
    let mut res = ProvableQueryResult::new_from_raw_data(
        1,
        Indexes::Sparse(vec![0]),
        vec![0b1111_1111_u8; 38],
    );
    res.data_mut()[37] = 0b0000_0001_u8;
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn evaluation_fails_if_integer_overflow_happens() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let binding = [i64::from(i32::MAX) + 1_i64, 11, 12];
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&binding)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::Int); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn evaluation_fails_if_data_is_missing() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let mut res = ProvableQueryResult::new(&indexes, &cols);
    *res.num_columns_mut() = 3;
    let evaluation_point = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut evaluation_vec = [Curve25519Scalar::ZERO; 4];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 4, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn we_can_convert_a_provable_result_to_a_final_result() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::BigInt(&[10, 11, 12])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(std::convert::Into::into).collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![10, 12]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_128_bits() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let cols: [Column<Curve25519Scalar>; 1] = [Column::Int128(&[10, 11, i128::MAX])];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![ColumnField::new("a1".parse().unwrap(), ColumnType::Int128)];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(std::convert::Into::into).collect();
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
fn we_can_convert_a_provable_result_to_a_final_result_with_252_bits() {
    let indexes = Indexes::Sparse(vec![0, 2]);
    let values = [
        Curve25519Scalar::from(10),
        Curve25519Scalar::from(11),
        Curve25519Scalar::MAX_SIGNED,
    ];

    let cols: [Column<Curve25519Scalar>; 1] = [Column::Scalar(&values)];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![ColumnField::new(
        "a1".parse().unwrap(),
        ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
    )];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(std::convert::Into::into).collect();
    let schema = Arc::new(Schema::new(column_fields));

    let expected_res = RecordBatch::try_new(
        schema,
        vec![Arc::new(
            Decimal256Array::from([i256::from(10), Curve25519Scalar::MAX_SIGNED.into()].to_vec())
                .with_precision_and_scale(75, 0)
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
    let values3 = ["abc", "fg", "de"];
    let scalars3 = values3
        .iter()
        .map(|v| Curve25519Scalar::from(*v))
        .collect::<Vec<_>>();
    let values4 = [
        Curve25519Scalar::from(10),
        Curve25519Scalar::from(11),
        Curve25519Scalar::MAX_SIGNED,
    ];

    let cols: [Column<Curve25519Scalar>; 4] = [
        Column::BigInt(&values1),
        Column::Int128(&values2),
        Column::VarChar((&values3, &scalars3)),
        Column::Scalar(&values4),
    ];
    let res = ProvableQueryResult::new(&indexes, &cols);
    let column_fields = vec![
        ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("a2".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("a3".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "a4".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields.iter().map(std::convert::Into::into).collect();
    let schema = Arc::new(Schema::new(column_fields));
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
            Arc::new(
                Decimal256Array::from(vec![i256::from(10), Curve25519Scalar::MAX_SIGNED.into()])
                    .with_precision_and_scale(75, 0)
                    .unwrap(),
            ),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}
