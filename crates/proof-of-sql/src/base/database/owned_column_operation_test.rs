use super::{ColumnOperationError, OwnedColumn, OwnedNullableColumn};
use crate::base::{
    database::{table::TableError, OwnedColumnError},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use alloc::{string::ToString, vec};

#[test]
fn test_from_table_error_for_owned_column_error() {
    let table_error = TableError::PresenceLengthMismatch;
    let owned_column_error = OwnedColumnError::from(table_error);

    match owned_column_error {
        OwnedColumnError::TableError { source } => {
            assert!(matches!(source, TableError::PresenceLengthMismatch));
        }
        _ => panic!("Expected TableError variant"),
    }
}

#[test]
fn test_tiny_int_element_wise_eq() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 3, 3]);

    let result = lhs.element_wise_eq(&rhs).unwrap();

    match result {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, vec![true, false, true]);
        }
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn test_small_int_element_wise_eq() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 3]);

    let result = lhs.element_wise_eq(&rhs).unwrap();

    match result {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, vec![true, false, true]);
        }
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn test_timestamp_tz_element_wise_eq() {
    let tu = PoSQLTimeUnit::Millisecond;
    let tz = PoSQLTimeZone::utc();
    let lhs = OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, vec![1, 3, 3]);

    let result = lhs.element_wise_eq(&rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn test_nullable_column_element_wise_and_both_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let lhs_presence = Some(vec![true, false, true, false]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
    let rhs_presence = Some(vec![true, false, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_and(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_and_left_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let lhs_presence = Some(vec![true, false, true, false]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
    let rhs = OwnedNullableColumn::new(rhs_values);

    let result = lhs.element_wise_and(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, true]);
}

#[test]
fn test_nullable_column_element_wise_and_right_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let lhs = OwnedNullableColumn::new(lhs_values);

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
    let rhs_presence = Some(vec![true, false, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_and(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, true, true, true]);
}

#[test]
fn test_nullable_column_element_wise_or_both_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let lhs_presence = Some(vec![true, false, true, false]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
    let rhs_presence = Some(vec![true, false, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_or(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, true, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_or_left_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let lhs_presence = Some(vec![true, false, true, false]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![false, true, false, true]);
    let rhs = OwnedNullableColumn::new(rhs_values);

    let result = lhs.element_wise_or(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, true, true, true]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, true, true, true]);
}

#[test]
fn test_nullable_column_element_wise_or_right_null() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![false, true, false, true]);
    let lhs = OwnedNullableColumn::new(lhs_values);

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let rhs_presence = Some(vec![true, false, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_or(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, true, true, true]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, true, true, true]);
}

#[test]
fn test_nullable_column_element_wise_eq_tiny_int() {
    let lhs_values = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3, 4]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 4, 4]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_eq(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_lt_small_int() {
    let lhs_values = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3, 4]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::SmallInt(vec![2, 1, 2, 4]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_lt(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_gt_int() {
    let lhs_values = OwnedColumn::<TestScalar>::Int(vec![2, 2, 3, 4]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Int(vec![1, 3, 4, 4]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_gt(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_add_big_int() {
    let lhs_values = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3, 4]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30, 40]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_add(&rhs).unwrap();

    match &result.values {
        OwnedColumn::BigInt(col) => {
            assert_eq!(col, &vec![11, 22, 33, 44]);
        }
        _ => panic!("Expected BigInt column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_sub_int128() {
    let lhs_values = OwnedColumn::<TestScalar>::Int128(vec![10, 20, 30, 40]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Int128(vec![1, 2, 3, 4]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_sub(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Int128(col) => {
            assert_eq!(col, &vec![9, 18, 27, 36]);
        }
        _ => panic!("Expected Int128 column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_mul_uint8() {
    let lhs_values = OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3, 4]);
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Uint8(vec![10, 20, 30, 40]);
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_mul(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Uint8(col) => {
            assert_eq!(col, &vec![10, 40, 90, 160]);
        }
        _ => panic!("Expected Uint8 column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_nullable_column_element_wise_div_decimal() {
    let precision = Precision::new(19).unwrap();
    let scale = 2;
    let lhs_values = OwnedColumn::<TestScalar>::Decimal75(
        precision,
        scale,
        vec![10.into(), 20.into(), 30.into(), 40.into()],
    );
    let lhs_presence = Some(vec![true, false, true, true]);
    let lhs = OwnedNullableColumn::with_presence(lhs_values, lhs_presence).unwrap();

    let rhs_values = OwnedColumn::<TestScalar>::Decimal75(
        precision,
        scale,
        vec![2.into(), 4.into(), 5.into(), 8.into()],
    );
    let rhs_presence = Some(vec![true, true, true, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_div(&rhs).unwrap();

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true, false]);
}

#[test]
fn test_element_wise_operation_errors() {
    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 0, 1]);

    let result = lhs.element_wise_and(&rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));

    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false]);
    let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);

    let result = lhs.element_wise_and(&rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::DifferentColumnLength { .. })
    ));

    let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 0, 1]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 1, 0]);

    let result = lhs.element_wise_and(&rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));

    let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string(), "b".to_string()]);
    let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["c".to_string(), "d".to_string()]);

    let result = lhs.element_wise_add(&rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn test_timestamp_tz_operations() {
    let tu = PoSQLTimeUnit::Millisecond;
    let tz = PoSQLTimeZone::utc();

    let lhs = OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, vec![1000, 2000, 3000]);
    let rhs = OwnedColumn::<TestScalar>::TimestampTZ(tu, tz, vec![500, 2000, 4000]);
    let lt_result = lhs.element_wise_lt(&rhs);
    assert!(matches!(
        lt_result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));

    let gt_result = lhs.element_wise_gt(&rhs);
    assert!(matches!(
        gt_result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn test_var_char_comparison() {
    let lhs = OwnedColumn::<TestScalar>::VarChar(vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ]);
    let rhs = OwnedColumn::<TestScalar>::VarChar(vec![
        "apple".to_string(),
        "berry".to_string(),
        "apple".to_string(),
    ]);

    let eq_result = lhs.element_wise_eq(&rhs);
    if let Err(err) = &eq_result {
        assert!(matches!(
            err,
            ColumnOperationError::BinaryOperationInvalidColumnType { .. }
        ));
    } else {
        match eq_result.unwrap() {
            OwnedColumn::Boolean(col) => {
                assert_eq!(col, vec![true, false, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    let lhs =
        OwnedColumn::<TestScalar>::VarBinary(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);
    let rhs =
        OwnedColumn::<TestScalar>::VarBinary(vec![vec![1, 2, 3], vec![1, 2, 3], vec![9, 8, 7]]);

    let eq_result = lhs.element_wise_eq(&rhs);
    assert!(matches!(
        eq_result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn test_mixed_nullable_column_operations() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
    let lhs = OwnedNullableColumn::new(lhs_values);

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]);
    let rhs_presence = Some(vec![true, false, true]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();
    let and_result = lhs.element_wise_and(&rhs).unwrap();

    match &and_result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![false, false, true]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(and_result.presence.is_some());
    let presence = and_result.presence.unwrap();
    assert_eq!(presence, vec![true, true, true]);

    let or_result = lhs.element_wise_or(&rhs).unwrap();

    match &or_result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, false, true]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(or_result.presence.is_some());
    let presence = or_result.presence.unwrap();
    assert_eq!(presence, vec![true, false, true]);
}

#[test]
fn test_three_valued_logic_edge_cases() {
    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![false, false]);
    let lhs = OwnedNullableColumn::new(lhs_values);

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, false]);
    let rhs_presence = Some(vec![false, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_and(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![false, false]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, true]);

    let lhs_values = OwnedColumn::<TestScalar>::Boolean(vec![true, true]);
    let lhs = OwnedNullableColumn::new(lhs_values);

    let rhs_values = OwnedColumn::<TestScalar>::Boolean(vec![false, true]);
    let rhs_presence = Some(vec![false, false]);
    let rhs = OwnedNullableColumn::with_presence(rhs_values, rhs_presence).unwrap();

    let result = lhs.element_wise_or(&rhs).unwrap();

    match &result.values {
        OwnedColumn::Boolean(col) => {
            assert_eq!(col, &vec![true, true]);
        }
        _ => panic!("Expected Boolean column"),
    }

    assert!(result.presence.is_some());
    let presence = result.presence.unwrap();
    assert_eq!(presence, vec![true, true]);
}

#[test]
fn test_decimal_operations() {
    let precision = Precision::new(19).unwrap();
    let scale = 2;

    let lhs = OwnedColumn::<TestScalar>::Decimal75(
        precision,
        scale,
        vec![10.into(), 20.into(), 30.into()],
    );

    let rhs = OwnedColumn::<TestScalar>::Decimal75(
        precision,
        scale,
        vec![5.into(), 10.into(), 15.into()],
    );

    let add_result = lhs.element_wise_add(&rhs).unwrap();
    let sub_result = lhs.element_wise_sub(&rhs).unwrap();
    let mul_result = lhs.element_wise_mul(&rhs).unwrap();
    let div_result = lhs.element_wise_div(&rhs).unwrap();

    assert!(matches!(add_result, OwnedColumn::Decimal75(_, _, _)));
    assert!(matches!(sub_result, OwnedColumn::Decimal75(_, _, _)));
    assert!(matches!(mul_result, OwnedColumn::Decimal75(_, _, _)));
    assert!(matches!(div_result, OwnedColumn::Decimal75(_, _, _)));
}
