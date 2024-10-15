#[cfg(feature = "arrow")]
#[test]
fn we_can_form_the_provable_query_result() {
    use crate::{
        base::{
            database::{Column, ColumnField, ColumnType},
            scalar::Curve25519Scalar,
        },
        sql::proof::ProvableQueryResult,
    };
    use alloc::sync::Arc;
    #[cfg(feature = "arrow")]
    use arrow::{
        array::Int64Array,
        datatypes::{Field, Schema},
        record_batch::RecordBatch,
    };

    let col1: Column<Curve25519Scalar> = Column::BigInt(&[11_i64, 12]);
    let col2: Column<Curve25519Scalar> = Column::BigInt(&[-3_i64, -4]);
    let res = ProvableQueryResult::new(2, &[col1, col2]);

    let column_fields = vec![
        ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
    ];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));

    let expected_res = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![11, 12])),
            Arc::new(Int64Array::from(vec![-3, -4])),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}
