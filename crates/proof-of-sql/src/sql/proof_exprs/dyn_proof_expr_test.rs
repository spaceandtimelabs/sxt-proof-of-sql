use crate::{
    base::database::{ColumnRef, ColumnType, TableRef},
    sql::{parse::ConversionError, proof_exprs::DynProofExpr},
};
use sqlparser::ast::Ident;

#[test]
fn test_try_new_is_null() {
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = DynProofExpr::try_new_is_null(column_expr).unwrap();

    match is_null_expr {
        DynProofExpr::IsNull(_) => {}
        _ => panic!("Expected IsNull expression"),
    }
}

#[test]
fn test_try_new_is_not_null() {
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_not_null_expr = DynProofExpr::try_new_is_not_null(column_expr).unwrap();

    match is_not_null_expr {
        DynProofExpr::IsNotNull(_) => {}
        _ => panic!("Expected IsNotNull expression"),
    }
}

#[test]
fn test_try_new_is_true_with_boolean() {
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = DynProofExpr::try_new_is_true(column_expr).unwrap();

    match is_true_expr {
        DynProofExpr::IsTrue(_) => {}
        _ => panic!("Expected IsTrue expression"),
    }
}

#[test]
fn test_try_new_is_true_with_non_boolean() {
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let result = DynProofExpr::try_new_is_true(column_expr);

    assert!(result.is_err());

    match result {
        Err(ConversionError::InvalidDataType { expected, actual }) => {
            assert_eq!(expected, ColumnType::Boolean);
            assert_eq!(actual, ColumnType::Int);
        }
        _ => panic!("Expected InvalidDataType error"),
    }
}
