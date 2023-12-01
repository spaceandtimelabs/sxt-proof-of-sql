use super::{filter_columns, fold_columns, fold_vals};
use crate::base::{database::Column, scalar::ArkScalar};
use bumpalo::Bump;

#[test]
fn we_can_filter_columns() {
    let selection = vec![true, false, true, false, true];
    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
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
        ]
    );
}
#[test]
fn we_can_filter_columns_with_empty_result() {
    let selection = vec![false, false, false, false, false];
    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
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
        ]
    );
}

#[test]
fn we_can_fold_columns() {
    let str_scalars = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [2.into(), 3.into(), 5.into(), 7.into(), 1.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9, 0]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
    ];
    let alloc = Bump::new();
    let result = fold_columns(&alloc, 77.into(), 10.into(), &columns, 5);
    assert_eq!(
        result,
        vec![
            ArkScalar::from(77 + 2061) + ArkScalar::from(100) * ArkScalar::from("1"),
            ArkScalar::from(77 + 3072) + ArkScalar::from(100) * ArkScalar::from("2"),
            ArkScalar::from(77 + 5083) + ArkScalar::from(100) * ArkScalar::from("3"),
            ArkScalar::from(77 + 7094) + ArkScalar::from(100) * ArkScalar::from("4"),
            ArkScalar::from(77 + 1005) + ArkScalar::from(100) * ArkScalar::from("5")
        ]
    );
}

#[test]
fn we_can_fold_columns_that_get_padded_with_0() {
    let str_scalars = ["1".into(), "2".into(), "3".into()];
    let scalars = [2.into(), 3.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9]),
        Column::VarChar((&["1", "2", "3"], &str_scalars)),
        Column::Scalar(&scalars),
    ];
    let alloc = Bump::new();
    let result = fold_columns(&alloc, 77.into(), 10.into(), &columns, 10);
    assert_eq!(
        result,
        vec![
            ArkScalar::from(77 + 2061) + ArkScalar::from(100) * ArkScalar::from("1"),
            ArkScalar::from(77 + 3072) + ArkScalar::from(100) * ArkScalar::from("2"),
            ArkScalar::from(77 + 83) + ArkScalar::from(100) * ArkScalar::from("3"),
            ArkScalar::from(77 + 94),
            ArkScalar::from(77 + 5),
            ArkScalar::from(77),
            ArkScalar::from(77),
            ArkScalar::from(77),
            ArkScalar::from(77),
            ArkScalar::from(77)
        ]
    );
}

#[test]
fn we_can_fold_empty_columns() {
    let columns = vec![
        Column::BigInt(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
    ];
    let alloc = Bump::new();
    let result = fold_columns(&alloc, 77.into(), 10.into(), &columns, 0);
    assert_eq!(result, vec![]);
}

#[test]
fn we_can_fold_vals() {
    assert_eq!(
        fold_vals(77.into(), 10.into(), [], 33.into(),),
        (77 * 33).into()
    );
    assert_eq!(
        fold_vals(
            77.into(),
            10.into(),
            [ArkScalar::from(1), 2.into(), 3.into(), 4.into(), 5.into()],
            33.into(),
        ),
        (77 * 33 + 54321).into()
    );
}
