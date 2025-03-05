use crate::{
    base::{
        database::{order_by_util::*, Column, ColumnType, OwnedColumn, TableOperationError},
        math::{decimal::Precision, non_negative_i32::NonNegativeI32},
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use core::cmp::Ordering;

#[test]
fn we_can_compare_indexes_by_columns_for_fixedsizebinary_hex_i32_full_suite() {
    // Each row is 4 bytes in big-endian format, representing:
    //  row0 => 0x00 0x00 0x00 0x00  (0)
    //  row1 => 0x00 0x00 0x00 0x01  (1)
    //  row2 => 0x00 0x00 0x00 0x02  (2)
    //  row3 => 0x7F 0xFF 0xFF 0xFF  (i32::MAX)
    let slice = &[
        0x00, 0x00, 0x00, 0x00, // row0
        0x00, 0x00, 0x00, 0x01, // row1
        0x00, 0x00, 0x00, 0x02, // row2
        0x7F, 0xFF, 0xFF, 0xFF, // row3
    ];

    let width = NonNegativeI32::new(4).expect("must not be negative");
    let col = Column::FixedSizeBinary(width, slice);

    let columns: &[Column<TestScalar>] = &[col];

    // row0 == row0
    assert_eq!(compare_indexes_by_columns(columns, 0, 0), Ordering::Equal);
    // row1 == row1
    assert_eq!(compare_indexes_by_columns(columns, 1, 1), Ordering::Equal);
    // row2 == row2
    assert_eq!(compare_indexes_by_columns(columns, 2, 2), Ordering::Equal);
    // row3 == row3
    assert_eq!(compare_indexes_by_columns(columns, 3, 3), Ordering::Equal);

    // row0 < row1
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    // row1 < row2
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    // row2 < row3
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);

    // row1 > row0
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Greater);
    // row2 > row1
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    // row3 > row2
    assert_eq!(compare_indexes_by_columns(columns, 3, 2), Ordering::Greater);

    // row0 <= row0 (Equal)
    {
        let ordering = compare_indexes_by_columns(columns, 0, 0);
        assert!(matches!(ordering, Ordering::Less | Ordering::Equal));
    }
    // row0 <= row1 (Less)
    {
        let ordering = compare_indexes_by_columns(columns, 0, 1);
        assert!(matches!(ordering, Ordering::Less | Ordering::Equal));
    }

    // row3 >= row2 (Greater)
    {
        let ordering = compare_indexes_by_columns(columns, 3, 2);
        assert!(matches!(ordering, Ordering::Greater | Ordering::Equal));
    }
    // row3 >= row3 (Equal)
    {
        let ordering = compare_indexes_by_columns(columns, 3, 3);
        assert!(matches!(ordering, Ordering::Greater | Ordering::Equal));
    }

    //
    // A couple more quick combos:
    //

    // row1 <= row2 => True, because row1 < row2
    {
        let ordering = compare_indexes_by_columns(columns, 1, 2);
        assert!(matches!(ordering, Ordering::Less | Ordering::Equal));
    }

    // row2 >= row1 => True, because row2 > row1
    {
        let ordering = compare_indexes_by_columns(columns, 2, 1);
        assert!(matches!(ordering, Ordering::Greater | Ordering::Equal));
    }
}

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
fn we_can_compare_owned_columns_with_direction_fixedsizebinary_and_others() {
    let col_small = OwnedColumn::SmallInt(vec![1, 1, 2, 1, 1]);

    let col_varchar = OwnedColumn::VarChar(
        ["b", "b", "a", "b", "a"]
            .iter()
            .map(ToString::to_string)
            .collect(),
    );

    let col_decimal = OwnedColumn::Decimal75(
        Precision::new(70).unwrap(),
        20,
        [-3, 2, 2, -3, 2]
            .iter()
            .map(|&n| TestScalar::from(n))
            .collect(),
    );

    let fsbin_slice = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x7F, 0xFF, 0xFF,
        0xFF, 0x00, 0x00, 0x00, 0x00,
    ];
    let width = NonNegativeI32::new(4).expect("width must be >= 0");
    let col_fsbin = OwnedColumn::FixedSizeBinary(width, fsbin_slice);

    let order_by_pairs = vec![
        (col_small, true),
        (col_varchar, false),
        (col_decimal, true),
        (col_fsbin, false),
    ];

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 1),
        Ordering::Less
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 2),
        Ordering::Less
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 3),
        Ordering::Greater
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 1, 3),
        Ordering::Greater
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 1, 1),
        Ordering::Equal
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 2, 4),
        Ordering::Greater
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 3, 4),
        Ordering::Less
    );

    assert_eq!(
        compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 4, 0),
        Ordering::Greater
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
