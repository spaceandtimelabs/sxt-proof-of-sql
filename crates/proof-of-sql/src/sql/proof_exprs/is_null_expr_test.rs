use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    sql::proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsNullExpr},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn test_is_null_expr() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(Default::default());
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Insert the column values into the table map
    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(Default::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    // Create a ColumnRef directly instead of trying to convert from Ident
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));
    let result = is_null_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // Values at index 1 and 3 should be NULL, so IS NULL should be true
            assert_eq!(values[0], false);
            assert_eq!(values[1], true);
            assert_eq!(values[2], false);
            assert_eq!(values[3], true);
            assert_eq!(values[4], false);
        }
        _ => panic!("Expected boolean column"),
    }
}
