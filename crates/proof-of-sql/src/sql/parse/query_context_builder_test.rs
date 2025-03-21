use crate::{
    base::{
        database::{ColumnType, TableRef, TestSchemaAccessor},
        map::IndexMap,
    },
    sql::parse::{query_context_builder::QueryContextBuilder, ConversionError},
};
use alloc::boxed::Box;
use proof_of_sql_parser::{
    intermediate_ast::{Expression, TableExpression},
    Identifier,
};
use sqlparser::ast::Ident;

fn setup_test_schema_accessor() -> TestSchemaAccessor {
    let tab_ref = TableRef::new("sxt", "sxt_tab");

    let mut schema_map = IndexMap::default();
    let mut column_types = IndexMap::default();

    column_types.insert(Ident::new("boolean_column"), ColumnType::Boolean);
    column_types.insert(Ident::new("varchar_column"), ColumnType::VarChar);
    schema_map.insert(tab_ref, column_types);

    TestSchemaAccessor::new(schema_map)
}

fn create_table_expr() -> Box<TableExpression> {
    let table_id = Identifier::try_new("sxt_tab").expect("Invalid table identifier");
    let schema_id = Identifier::try_new("sxt").expect("Invalid schema identifier");

    Box::new(TableExpression::Named {
        table: table_id,
        schema: Some(schema_id),
    })
}

#[test]
fn test_is_true_expression_with_non_boolean_input() {
    let schema_accessor = setup_test_schema_accessor();
    let builder = QueryContextBuilder::new(&schema_accessor);
    let table_exprs = [create_table_expr()];
    let builder = builder.visit_table_expr(&table_exprs, Ident::new("default"));
    let column_id = Identifier::try_new("varchar_column").expect("Invalid identifier");
    let string_expr = Expression::Column(column_id);
    let is_true_expr = Expression::IsTrue(Box::new(string_expr));
    let result = builder.visit_where_expr(Some(Box::new(is_true_expr)));

    assert!(
        result.is_err(),
        "Expected an error for non-boolean input to IsTrue"
    );
    match result {
        Err(ConversionError::InvalidDataType { expected, actual }) => {
            assert_eq!(expected, ColumnType::Boolean);
            assert_eq!(actual, ColumnType::VarChar);
        }
        _other => panic!("Expected InvalidDataType error, got something else"),
    }
}

#[test]
fn test_is_true_expression_with_boolean_input() {
    let schema_accessor = setup_test_schema_accessor();
    let builder = QueryContextBuilder::new(&schema_accessor);
    let table_exprs = [create_table_expr()];
    let builder = builder.visit_table_expr(&table_exprs, Ident::new("default"));
    let column_id = Identifier::try_new("boolean_column").expect("Invalid identifier");
    let bool_expr = Expression::Column(column_id);
    let is_true_expr = Expression::IsTrue(Box::new(bool_expr));
    let result = builder.visit_where_expr(Some(Box::new(is_true_expr)));

    assert!(
        result.is_ok(),
        "Expected IsTrue with boolean input to succeed"
    );
}
