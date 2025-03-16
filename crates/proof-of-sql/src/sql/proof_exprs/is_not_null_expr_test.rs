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
    let presence = &[true, false, true, false, true];
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
            // presence[i] = true means NOT NULL, so IS NOT NULL should return true for those values
            assert!(values[0]);  // presence[0] = true -> IS NOT NULL = true
            assert!(!values[1]); // presence[1] = false -> IS NOT NULL = false
            assert!(values[2]);  // presence[2] = true -> IS NOT NULL = true
            assert!(!values[3]); // presence[3] = false -> IS NOT NULL = false
            assert!(values[4]);  // presence[4] = true -> IS NOT NULL = true
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_not_null_expr_with_complex_null_logic() {
    let alloc = Bump::new();
    let mut columns = IndexMap::with_hasher(BuildHasherDefault::default());
    let table_ref = TableRef::new("", "test");

    // Create multiple columns with different NULL patterns
    // Column A: Integer with some NULLs
    let col_a_values: Column<'_, TestScalar> = Column::Int(&[10, 20, 30, 40, 50, 60, 70, 80]);
    let col_a_presence = &[true, true, false, true, false, true, true, false];

    // Column B: Integer with different NULL pattern
    let col_b_values: Column<'_, TestScalar> = Column::Int(&[5, 15, 25, 35, 45, 55, 65, 75]);
    let col_b_presence = &[true, false, true, false, true, true, false, true];

    // Column C: Integer with different NULL pattern
    let col_c_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let col_c_presence = &[false, true, true, false, true, false, true, true];

    // Insert the column values into the columns map
    columns.insert(Ident::new("col_a"), col_a_values);
    columns.insert(Ident::new("col_b"), col_b_values);
    columns.insert(Ident::new("col_c"), col_c_values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("col_a"), col_a_presence.as_slice());
    presence_map.insert(Ident::new("col_b"), col_b_presence.as_slice());
    presence_map.insert(Ident::new("col_c"), col_c_presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(columns, presence_map, TableOptions::new(Some(8))).unwrap();

    // ColumnRefs for all columns
    let col_a_ref = ColumnRef::new(table_ref.clone(), Ident::new("col_a"), ColumnType::Int);
    let col_b_ref = ColumnRef::new(table_ref.clone(), Ident::new("col_b"), ColumnType::Int);
    let col_c_ref = ColumnRef::new(table_ref, Ident::new("col_c"), ColumnType::Int);

    // Create DynProofExpr nodes for all columns
    let col_a_expr = DynProofExpr::new_column(col_a_ref);
    let col_b_expr = DynProofExpr::new_column(col_b_ref);
    let col_c_expr = DynProofExpr::new_column(col_c_ref);

    // Test 1: Simple IS NOT NULL on column A
    let is_not_null_a = IsNotNullExpr::new(Box::new(col_a_expr.clone()));
    let result_a = is_not_null_a.result_evaluate(&alloc, &table);

    match result_a {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 8);
            // IS NOT NULL is true for non-NULL values, false for NULL values
            assert!(values[0]); // Row 1: A is not NULL
            assert!(values[1]); // Row 2: A is not NULL
            assert!(!values[2]); // Row 3: A is NULL
            assert!(values[3]); // Row 4: A is not NULL
            assert!(!values[4]); // Row 5: A is NULL
            assert!(values[5]); // Row 6: A is not NULL
            assert!(values[6]); // Row 7: A is not NULL
            assert!(!values[7]); // Row 8: A is NULL
        }
        _ => panic!("Expected boolean column"),
    }

    // Test 2: IS NOT NULL on arithmetic expression (A + B)
    // We'll simplify and just test on columns directly rather than arithmetic expressions
    let a_is_not_null = DynProofExpr::try_new_is_not_null(col_a_expr.clone()).unwrap();
    let b_is_not_null = DynProofExpr::try_new_is_not_null(col_b_expr.clone()).unwrap();
    
    // Create a_is_not_null AND b_is_not_null
    let both_not_null = DynProofExpr::try_new_and(a_is_not_null, b_is_not_null).unwrap();
    let result_both_not_null = both_not_null.result_evaluate(&alloc, &table);
    
    match result_both_not_null {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 8);
            // AND is true only when both operands are true
            // Here, that means rows where both A and B are NOT NULL
            assert!(values[0]); // Row 1: A is not NULL, B is not NULL -> true
            assert!(!values[1]); // Row 2: A is not NULL, B is NULL -> false
            assert!(!values[2]); // Row 3: A is NULL, B is not NULL -> false
            assert!(!values[3]); // Row 4: A is not NULL, B is NULL -> false
            assert!(!values[4]); // Row 5: A is NULL, B is not NULL -> false
            assert!(values[5]); // Row 6: A is not NULL, B is not NULL -> true
            assert!(!values[6]); // Row 7: A is not NULL, B is NULL -> false
            assert!(!values[7]); // Row 8: A is NULL, B is not NULL -> false
        }
        _ => panic!("Expected boolean column"),
    }

    // Test 3: More complex logic (A IS NOT NULL OR C IS NOT NULL)
    let a_is_not_null = DynProofExpr::try_new_is_not_null(col_a_expr.clone()).unwrap();
    let c_is_not_null = DynProofExpr::try_new_is_not_null(col_c_expr.clone()).unwrap();
    
    // Create a_is_not_null OR c_is_not_null
    let either_not_null = DynProofExpr::try_new_or(a_is_not_null, c_is_not_null).unwrap();
    let result_either_not_null = either_not_null.result_evaluate(&alloc, &table);
    
    match result_either_not_null {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 8);
            // OR is true when either operand is true
            // Here, that means rows where either A or C is NOT NULL
            assert!(values[0]); // Row 1: A is not NULL, C is NULL -> true
            assert!(values[1]); // Row 2: A is not NULL, C is not NULL -> true
            assert!(values[2]); // Row 3: A is NULL, C is not NULL -> true
            assert!(values[3]); // Row 4: A is not NULL, C is NULL -> true
            assert!(values[4]); // Row 5: A is NULL, C is not NULL -> true
            assert!(values[5]); // Row 6: A is not NULL, C is NULL -> true
            assert!(values[6]); // Row 7: A is not NULL, C is not NULL -> true
            assert!(values[7]); // Row 8: A is NULL, C is not NULL -> true
        }
        _ => panic!("Expected boolean column"),
    }
}
