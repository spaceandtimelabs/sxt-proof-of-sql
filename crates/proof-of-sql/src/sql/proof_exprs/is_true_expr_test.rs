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

#[test]
fn test_is_true_expr() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(Default::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(Default::default());
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
            assert_eq!(values[0], true); // true and not NULL
            assert_eq!(values[1], false); // NULL
            assert_eq!(values[2], true); // true and not NULL
            assert_eq!(values[3], false); // NULL
            assert_eq!(values[4], true); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_false_values() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(Default::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, false, true, false, false];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(Default::default());
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
            assert_eq!(values[0], true); // true and not NULL (presence[0] = true, values[0] = true)
            assert_eq!(values[1], false); // NULL (presence[1] = false)
            assert_eq!(values[2], true); // true and not NULL (presence[2] = true, values[2] = true)
            assert_eq!(values[3], false); // NULL (presence[3] = false)
            assert_eq!(values[4], false); // NULL (presence[4] = false)
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(Default::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(Default::default());
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
            assert_eq!(values[0], true); // true and not NULL
            assert_eq!(values[1], false); // NULL
            assert_eq!(values[2], true); // true and not NULL
            assert_eq!(values[3], false); // NULL
            assert_eq!(values[4], true); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_non_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(Default::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, true, false, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(Default::default());
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
            assert_eq!(values[0], true); // true and not NULL
            assert_eq!(values[1], false); // false and not NULL
            assert_eq!(values[2], false); // NULL
            assert_eq!(values[3], false); // NULL
            assert_eq!(values[4], false); // false and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}
