use super::{filter_columns, fold_vals};
use crate::{
    base::{database::Column, math::precision::Precision, scalar::ArkScalar},
    sql::ast::dense_filter_util::fold_columns,
};
use bumpalo::Bump;
use num_traits::Zero;

#[test]
fn we_can_filter_columns() {
    let selection = vec![true, false, true, false, true];
    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), 0, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[1, 3, 5]),
            Column::Int128(&[1, 3, 5]),
            Column::VarChar((&["1", "3", "5"], &["1".into(), "3".into(), "5".into()])),
            Column::Scalar(&[1.into(), 3.into(), 5.into()]),
            Column::Decimal75(
                Precision::new(75).unwrap(),
                0,
                &[1.into(), 3.into(), 5.into()]
            )
        ]
    );
}
#[test]
fn we_can_filter_columns_with_empty_result() {
    let selection = vec![false, false, false, false, false];
    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}
#[test]
fn we_can_filter_empty_columns() {
    let selection = vec![];
    let columns = vec![
        Column::BigInt(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}

#[test]
fn we_can_fold_columns_with_scalars() {
    let expected = vec![
        ArkScalar::from(77 + 2061 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("1"),
        ArkScalar::from(77 + 3072 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("2"),
        ArkScalar::from(77 + 5083 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("3"),
        ArkScalar::from(77 + 7094 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("4"),
        ArkScalar::from(77 + 1005 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("5"),
    ];

    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [2.into(), 3.into(), 5.into(), 7.into(), 1.into()];
    let mut columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9, 0]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
    ];

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_columns_with_that_get_padded() {
    let expected = vec![
        ArkScalar::from(77 + 2061 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("1"),
        ArkScalar::from(77 + 3072 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("2"),
        ArkScalar::from(77 + 83 * 33) + ArkScalar::from(100 * 33) * ArkScalar::from("3"),
        ArkScalar::from(77 + 94 * 33),
        ArkScalar::from(77 + 5 * 33),
        ArkScalar::from(77),
        ArkScalar::from(77),
        ArkScalar::from(77),
        ArkScalar::from(77),
        ArkScalar::from(77),
        ArkScalar::from(77),
    ];

    let str_scalars = ["1".into(), "2".into(), "3".into()];
    let scalars = [2.into(), 3.into()];
    let mut columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9]),
        Column::VarChar((&["1", "2", "3"], &str_scalars)),
        Column::Scalar(&scalars),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_empty_columns() {
    let columns = vec![
        Column::BigInt(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(0, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);
    assert_eq!(result, vec![]);
}

#[test]
fn we_can_fold_vals() {
    assert_eq!(fold_vals(10.into(), &[]), Zero::zero());
    assert_eq!(
        fold_vals(
            10.into(),
            &[ArkScalar::from(1), 2.into(), 3.into(), 4.into(), 5.into()]
        ),
        (54321).into()
    );
}
