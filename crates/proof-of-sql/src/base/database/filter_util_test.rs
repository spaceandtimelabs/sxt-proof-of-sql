use crate::base::{
    database::{filter_util::*, Column, ColumnTypeAssociatedData},
    math::decimal::Precision,
    scalar::Curve25519Scalar,
};
use bumpalo::Bump;

#[test]
fn we_can_filter_columns() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let selection = vec![true, false, true, false, true];
    let str_scalars: [Curve25519Scalar; 5] =
        ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(meta, &[1, 2, 3, 4, 5]),
        Column::Int128(meta, &[1, 2, 3, 4, 5]),
        Column::VarChar(meta, (&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(meta, &scalars),
        Column::Decimal75(meta, Precision::new(75).unwrap(), 0, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(
        result,
        vec![
            Column::BigInt(meta, &[1, 3, 5]),
            Column::Int128(meta, &[1, 3, 5]),
            Column::VarChar(
                meta,
                (&["1", "3", "5"], &["1".into(), "3".into(), "5".into()])
            ),
            Column::Scalar(meta, &[1.into(), 3.into(), 5.into()]),
            Column::Decimal75(
                meta,
                Precision::new(75).unwrap(),
                0,
                &[1.into(), 3.into(), 5.into()]
            )
        ]
    );
}
#[test]
fn we_can_filter_columns_with_empty_result() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let selection = vec![false, false, false, false, false];
    let str_scalars: [Curve25519Scalar; 5] =
        ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(meta, &[1, 2, 3, 4, 5]),
        Column::Int128(meta, &[1, 2, 3, 4, 5]),
        Column::VarChar(meta, (&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(meta, &scalars),
        Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(meta, &[]),
            Column::Int128(meta, &[]),
            Column::VarChar(meta, (&[], &[])),
            Column::Scalar(meta, &[]),
            Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &[])
        ]
    );
}
#[test]
fn we_can_filter_empty_columns() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let selection = vec![];
    let columns = vec![
        Column::<Curve25519Scalar>::BigInt(meta, &[]),
        Column::Int128(meta, &[]),
        Column::VarChar(meta, (&[], &[])),
        Column::Scalar(meta, &[]),
        Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(meta, &[]),
            Column::Int128(meta, &[]),
            Column::VarChar(meta, (&[], &[])),
            Column::Scalar(meta, &[]),
            Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &[])
        ]
    );
}
