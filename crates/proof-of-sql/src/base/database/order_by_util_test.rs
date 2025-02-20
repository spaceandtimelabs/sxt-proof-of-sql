use crate::{
    base::{
        database::{Column, ColumnType, OwnedColumn, TableOperationError, order_by_util::*},
        math::decimal::Precision,
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
fn we_can_compare_single_row_of_tables() {
    let left_slice_a = &[55, 44, 44, 66, 66, 77, 66, 66, 66, 66];
    let left_slice_b = &[22, 44, 55, 44, 33, 22, 22, 11, 22, 22];
    let left_slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let left_column_a = Column::BigInt::<TestScalar>(left_slice_a);
    let left_column_b = Column::BigInt::<TestScalar>(left_slice_b);
    let left_column_c = Column::BigInt::<TestScalar>(left_slice_c);
    let left = &[left_column_a, left_column_b, left_column_c];

    let right_slice_a = &[77, 44, 66, 44, 77, 77, 66, 66, 55, 66];
    let right_slice_b = &[22, 55, 11, 77, 33, 33, 22, 22, 22, 11];
    let right_slice_c = &[11, 55, 22, 0, 77, 11, 33, 55, 11, 22];
    let right_column_a = Column::BigInt::<TestScalar>(right_slice_a);
    let right_column_b = Column::BigInt::<TestScalar>(right_slice_b);
    let right_column_c = Column::BigInt::<TestScalar>(right_slice_c);
    let right = &[right_column_a, right_column_b, right_column_c];

    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 1).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 1, 2).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 2, 3).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 2, 1).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 5, 0).unwrap(),
        Ordering::Equal
    );
}

#[test]
fn we_cannot_compare_single_row_of_tables_if_type_mismatch() {
    let left_slice = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let right_slice = &[
        true, false, true, true, false, true, false, true, false, true,
    ];
    let left_column = Column::BigInt::<TestScalar>(left_slice);
    let right_column = Column::Boolean::<TestScalar>(right_slice);
    let left = &[left_column];
    let right = &[right_column];
    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 1),
        Err(TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::Boolean
        })
    );
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
        [-3, 2, 2, -3, 2]
            .iter()
            .map(|&i| TestScalar::from(i))
            .collect(),
    );
    let order_by_pairs = vec![(col1, true), (col2, false), (col3, true)];
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

#[test]
fn we_can_compare_indexes_by_columns_for_varbinary_columns() {
    let raw_bytes = [
        b"foo".as_ref(),
        b"bar".as_ref(),
        b"baz".as_ref(),
        b"baz".as_ref(),
        b"bar".as_ref(),
    ];
    let scalars: Vec<TestScalar> = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect();
    let col_varbinary = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));
    let columns = &[col_varbinary];

    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater); // "foo" vs "bar"
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less); // "bar" vs "baz"
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal); // "baz" vs "baz"
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater); // "baz" vs "bar"
    assert_eq!(compare_indexes_by_columns(columns, 1, 4), Ordering::Equal); // "bar" vs "bar"
}
