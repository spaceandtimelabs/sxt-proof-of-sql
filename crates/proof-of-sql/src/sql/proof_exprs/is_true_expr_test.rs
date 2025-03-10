use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    sql::proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsTrueExpr},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use sqlparser::ast::Ident;
use std::hash::BuildHasherDefault;

#[test]
fn test_is_true_expr() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // NULL
            assert!(values[2]); // true and not NULL
            assert!(!values[3]); // NULL
            assert!(values[4]); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_false_values() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, false, true, false, false];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL (presence[0] = true, values[0] = true)
            assert!(!values[1]); // NULL (presence[1] = false)
            assert!(values[2]); // true and not NULL (presence[2] = true, values[2] = true)
            assert!(!values[3]); // NULL (presence[3] = false)
            assert!(!values[4]); // NULL (presence[4] = false)
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // NULL
            assert!(values[2]); // true and not NULL
            assert!(!values[3]); // NULL
            assert!(values[4]); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_non_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, true, false, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // false and not NULL
            assert!(!values[2]); // NULL
            assert!(!values[3]); // NULL
            assert!(!values[4]); // false and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}
