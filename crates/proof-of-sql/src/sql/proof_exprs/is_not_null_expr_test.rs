use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    sql::proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsNotNullExpr},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use sqlparser::ast::Ident;
use std::hash::BuildHasherDefault;

#[test]
fn test_is_not_null_expr() {
    let alloc = Bump::new();
    let mut columns = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence = &[false, true, false, true, false];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Insert the column values into the columns map
    columns.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(columns, presence_map, TableOptions::new(Some(5))).unwrap();

    // Create a ColumnRef directly instead of trying to convert from Ident
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_not_null_expr = IsNotNullExpr::new(Box::new(column_expr));

    // Evaluate the expression
    let result = is_not_null_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // Values at index 1 and 3 should be NULL (presence[i] = true), so IS NOT NULL should be false
            assert!(values[0]);
            assert!(!values[1]);
            assert!(values[2]);
            assert!(!values[3]);
            assert!(values[4]);
        }
        _ => panic!("Expected boolean column"),
    }
}
