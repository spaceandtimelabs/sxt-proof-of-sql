use super::{fold_columns, fold_vals};
use crate::base::{database::Column, math::decimal::Precision, scalar::Curve25519Scalar};
use bumpalo::Bump;
use num_traits::Zero;
use crate::base::database::ColumnTypeAssociatedData;

#[test]
fn we_can_fold_columns_with_scalars() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let expected = vec![
        Curve25519Scalar::from(77 + 2061 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("1"),
        Curve25519Scalar::from(77 + 3072 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("2"),
        Curve25519Scalar::from(77 + 5083 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("3"),
        Curve25519Scalar::from(77 + 7094 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("4"),
        Curve25519Scalar::from(77 + 1005 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("5"),
    ];

    let str_scalars: [Curve25519Scalar; 5] =
        ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [2.into(), 3.into(), 5.into(), 7.into(), 1.into()];
    let mut columns = vec![
        Column::BigInt(meta, &[1, 2, 3, 4, 5]),
        Column::Int128(meta, &[6, 7, 8, 9, 0]),
        Column::VarChar(meta, (&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(meta, &scalars),
    ];

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_columns_with_that_get_padded() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let expected = vec![
        Curve25519Scalar::from(77 + 2061 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("1"),
        Curve25519Scalar::from(77 + 3072 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("2"),
        Curve25519Scalar::from(77 + 83 * 33)
            + Curve25519Scalar::from(100 * 33) * Curve25519Scalar::from("3"),
        Curve25519Scalar::from(77 + 94 * 33),
        Curve25519Scalar::from(77 + 5 * 33),
        Curve25519Scalar::from(77),
        Curve25519Scalar::from(77),
        Curve25519Scalar::from(77),
        Curve25519Scalar::from(77),
        Curve25519Scalar::from(77),
        Curve25519Scalar::from(77),
    ];

    let str_scalars: [Curve25519Scalar; 3] = ["1".into(), "2".into(), "3".into()];
    let scalars = [2.into(), 3.into()];
    let mut columns = vec![
        Column::BigInt(meta, &[1, 2, 3, 4, 5]),
        Column::Int128(meta, &[6, 7, 8, 9]),
        Column::VarChar(meta, (&["1", "2", "3"], &str_scalars)),
        Column::Scalar(meta, &scalars),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_empty_columns() {
    let meta = ColumnTypeAssociatedData::NOT_NULLABLE;
    let columns = vec![
        Column::BigInt::<Curve25519Scalar>(meta, &[]),
        Column::Int128(meta, &[]),
        Column::VarChar(meta, (&[], &[])),
        Column::Scalar(meta, &[]),
        Column::Decimal75(meta, Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(0, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);
    assert_eq!(result, vec![]);
}

#[test]
fn we_can_fold_vals() {
    assert_eq!(fold_vals(Curve25519Scalar::from(10), &[]), Zero::zero());
    assert_eq!(
        fold_vals(
            10.into(),
            &[
                Curve25519Scalar::from(1),
                2.into(),
                3.into(),
                4.into(),
                5.into()
            ]
        ),
        (54321).into()
    );
}
