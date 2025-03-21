use crate::{
    base::{
        database::{ColumnRef, ColumnType, TableRef},
        map::indexmap,
    },
    sql::{parse::FilterExecBuilder, proof_exprs::DynProofExpr},
};
use alloc::boxed::Box;
use proof_of_sql_parser::{
    intermediate_ast::{BinaryOperator, Expression, Literal},
    Identifier,
};
use sqlparser::ast::Ident;

fn test_table_ref() -> TableRef {
    TableRef::new("test", "table")
}

#[allow(clippy::similar_names)]
fn setup_test_builder() -> FilterExecBuilder {
    let col_a_id: Ident = "col_a".into();
    let col_b_id: Ident = "col_b".into();
    let col_c_id: Ident = "col_c".into();

    let table_ref = test_table_ref();

    let column_mapping = indexmap! {
        col_a_id => ColumnRef::new(table_ref.clone(), "col_a".into(), ColumnType::BigInt),
        col_b_id => ColumnRef::new(table_ref.clone(), "col_b".into(), ColumnType::VarChar),
        col_c_id => ColumnRef::new(table_ref.clone(), "col_c".into(), ColumnType::Boolean),
    };

    let builder = FilterExecBuilder::new(column_mapping);
    builder.add_table_expr(table_ref)
}

fn create_is_null_ast_expr() -> Expression {
    let column_id = Identifier::try_new("col_a").expect("Invalid identifier");
    let column_expr = Box::new(Expression::Column(column_id));

    Expression::IsNull(column_expr)
}

fn create_or_with_null_check_ast_expr() -> Expression {
    let is_null_expr = Box::new(create_is_null_ast_expr());
    let bool_expr = Box::new(Expression::Literal(Literal::Boolean(true)));

    Expression::Binary {
        op: BinaryOperator::Or,
        left: is_null_expr,
        right: bool_expr,
    }
}

fn create_simple_boolean_expr() -> Expression {
    Expression::Literal(Literal::Boolean(true))
}

#[test]
fn test_or_expression_with_null_check() {
    let builder = setup_test_builder();
    let builder = builder
        .add_where_expr(Some(Box::new(create_or_with_null_check_ast_expr())))
        .unwrap();
    let filter_exec = builder.build();

    match &filter_exec.where_clause {
        DynProofExpr::Or(_) => {}
        DynProofExpr::IsTrue(_) => {
            panic!("OR expression with NULL check was incorrectly wrapped in IsTrueExpr");
        }
        _ => {
            panic!("Unexpected expression type");
        }
    }
}

#[test]
fn test_simple_boolean_expression() {
    let builder = setup_test_builder();
    let builder_with_expr = builder
        .add_where_expr(Some(Box::new(create_simple_boolean_expr())))
        .unwrap();
    let filter_exec = builder_with_expr.build();

    match &filter_exec.where_clause {
        DynProofExpr::IsTrue(_) => {}
        _ => {
            panic!("Expected the boolean literal to be wrapped in IsTrueExpr");
        }
    }
}
