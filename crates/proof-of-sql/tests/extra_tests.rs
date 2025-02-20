#![cfg(test)]
//! This file is added to improve test coverage of various modules in proof-of-sql.

use proof_of_sql::base::{
    commitment::Bounds,
    database::{
        Column,
        ColumnType,
        owned_table_utility::*,
        OwnedTable,
        OwnedColumn,
    },
    scalar::Curve25519Scalar,
};
use indexmap::IndexMap;
use sqlparser::ast::Ident;
use ahash::AHasher;
use std::hash::BuildHasherDefault;

type TestColumn<'a> = Column<'a, Curve25519Scalar>;

// ---------- Test for Column methods ----------
#[test]
fn test_column_len_and_is_empty() {
    // BigInt column.
    let col_bigint: TestColumn = Column::BigInt(&[1, 2, 3]);
    assert_eq!(col_bigint.len(), 3);
    assert!(!col_bigint.is_empty());
    
    // VarChar column, ensuring both slices have same length.
    let strings = ["h".into(), "w".into()];  // Create longer-lived strings
    let col_varchar: TestColumn = Column::VarChar((&["hello", "world"], &strings));
    assert_eq!(col_varchar.len(), 2);
    assert!(!col_varchar.is_empty());

    // Empty column
    let empty_col: TestColumn = Column::BigInt(&[]);
    assert_eq!(empty_col.len(), 0);
    assert!(empty_col.is_empty());
}

// ---------- Test for Database Column Operations ----------
#[test]
fn test_database_column_operations() {
    // Test column type checks
    let col: TestColumn = Column::BigInt(&[1, 2, 3]);
    
    // Test column metadata
    assert_eq!(col.column_type(), ColumnType::BigInt);
    assert_eq!(col.len(), 3);
    assert!(!col.is_empty());

    // Test different column types
    let bool_col: TestColumn = Column::Boolean(&[true, false, true]);
    assert_eq!(bool_col.column_type(), ColumnType::Boolean);

    let int_col: TestColumn = Column::Int(&[10, 20, 30]);
    assert_eq!(int_col.column_type(), ColumnType::Int);

    let small_int_col: TestColumn = Column::SmallInt(&[1, 2, 3]);
    assert_eq!(small_int_col.column_type(), ColumnType::SmallInt);
}

// ---------- Test for Commitment Column Bounds ----------
#[test]
fn test_commitment_column_bounds() {
    // Test valid bounds creation with i64 type which implements Ord
    let bounds = Bounds::<i64>::sharp(0, 100).unwrap();
    match bounds {
        Bounds::Sharp(inner) => {
            assert_eq!(inner.min(), &0);
            assert_eq!(inner.max(), &100);
        },
        _ => panic!("Expected Sharp bounds"),
    }
    
    // Test invalid bounds
    assert!(Bounds::<i64>::sharp(100, 0).is_err());

    // Test bounded bounds
    let bounded = Bounds::<i64>::bounded(0, 100).unwrap();
    match bounded {
        Bounds::Bounded(inner) => {
            assert_eq!(inner.min(), &0);
            assert_eq!(inner.max(), &100);
        },
        _ => panic!("Expected Bounded bounds"),
    }
}

// ---------- Test for Table Operations ----------
#[test]
fn test_table_operations() {
    // Test non-empty table
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1, 2, 3]),
        varchar("b", ["x", "y", "z"]),
    ]);
    assert_eq!(table.num_rows(), 3);
    assert!(!table.is_empty());
    assert_eq!(table.num_columns(), 2);

    // Test empty table (no columns)
    let empty_map: IndexMap<Ident, OwnedColumn<Curve25519Scalar>, BuildHasherDefault<AHasher>> = IndexMap::default();
    let empty_table: OwnedTable<Curve25519Scalar> = OwnedTable::try_new(empty_map).unwrap();
    assert_eq!(empty_table.num_rows(), 0);
    assert!(empty_table.is_empty());
    assert_eq!(empty_table.num_columns(), 0);
}

// End of extra coverage tests.