use crate::{
    base::{
        database::{order_by_util::*, Column, OwnedColumn},
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use core::cmp::Ordering;

#[test]
fn we_can_compare_indexes_by_columns_with_no_columns() {
    let columns: &[Column<TestScalar>; 0] = &[];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 2), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_bigint_columns() {
    let slice_a = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let column_a = Column::BigInt::<DoryScalar>(slice_a);
    let column_b = Column::BigInt::<DoryScalar>(slice_b);
    let column_c = Column::BigInt::<DoryScalar>(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_mixed_columns() {
    let slice_a = &["55", "44", "66", "66", "66", "77", "66", "66", "66", "66"];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let scals_a: Vec<TestScalar> = slice_a.iter().map(core::convert::Into::into).collect();
    let column_a = Column::VarChar((slice_a, &scals_a));
    let column_b = Column::Int128(slice_b);
    let column_c = Column::BigInt(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_owned_columns_for_mixed_columns() {
    let slice_a = ["55", "44", "66", "66", "66", "77", "66", "66", "66", "66"]
        .into_iter()
        .map(Into::into)
        .collect();
    let slice_b = vec![22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = vec![11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let column_a = OwnedColumn::<DoryScalar>::VarChar(slice_a);
    let column_b = OwnedColumn::Int128(slice_b);
    let column_c = OwnedColumn::BigInt(slice_c);

    let columns = &[&column_a];
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 1, 2),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 3),
        Ordering::Equal
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 1, 0),
        Ordering::Less
    );
    let columns = &[&column_a, &column_b];
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 1, 2),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 3),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 3, 4),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 7),
        Ordering::Equal
    );
    let columns = &[&column_a, &column_b, &column_c];
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 1, 2),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 3),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 3, 4),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 2, 7),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_owned_columns(columns, 6, 9),
        Ordering::Equal
    );
}

#[test]
fn we_can_compare_indexes_by_columns_for_scalar_columns() {
    let slice_a = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let scals_a: Vec<TestScalar> = slice_a.iter().map(core::convert::Into::into).collect();
    let column_a = Column::Scalar(&scals_a);
    let column_b = Column::Int128(slice_b);
    let column_c = Column::BigInt(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_columns_with_direction() {
    let col1: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 1, 2, 1, 1]);
    let col2: OwnedColumn<TestScalar> = OwnedColumn::VarChar(
        ["b", "b", "a", "b", "a"]
            .iter()
            .map(ToString::to_string)
            .collect(),
    );
    let col3: OwnedColumn<TestScalar> = OwnedColumn::Decimal75(
        Precision::new(70).unwrap(),
        20,
        [1, 2, 2, 1, 2]
            .iter()
            .map(|&i| TestScalar::from(i))
            .collect(),
    );
    let order_by_pairs = vec![
        (col1, OrderByDirection::Asc),
        (col2, OrderByDirection::Desc),
        (col3, OrderByDirection::Asc),
    ];
    // Equal on col1 and col2, less on col3
    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 1),
        Ordering::Less
    );
    // Less on col1
    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 2),
        Ordering::Less
    );
    // Equal on all 3 columns
    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 3),
        Ordering::Equal
    );
    // Equal on col1, greater on col2 reversed
    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 1, 4),
        Ordering::Less
    );
}
