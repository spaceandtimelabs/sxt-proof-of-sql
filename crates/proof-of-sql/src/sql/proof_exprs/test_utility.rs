use crate::{
    base::{
        database::{ColumnRef, LiteralValue, SchemaAccessor, TableRef},
        scalar::Scalar,
    },
    sql::proof_exprs::{
        add_subtract_expr::AddSubtractExpr, and_expr::AndExpr, column_expr::ColumnExpr,
        equals_expr::EqualsExpr, inequality_expr::InequalityExpr, literal_expr::LiteralExpr,
        multiply_expr::MultiplyExpr, not_expr::NotExpr, or_expr::OrExpr, DynProofExpr,
    },
};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Creates a new `DynProofExpr::Column` expression.
pub fn column<S: Scalar, A: SchemaAccessor>(
    table_ref: &TableRef,
    column_id: &str,
    accessor: &A,
) -> DynProofExpr {
    let column_id = column_id.parse().unwrap();
    let column_type = accessor
        .lookup_column(table_ref.clone(), column_id.clone())
        .unwrap();
    let column_ref = ColumnRef::new(table_ref.clone(), column_id, column_type);
    DynProofExpr::Column(ColumnExpr::new(column_ref))
}

/// Creates a new `DynProofExpr::Equal` expression.
pub fn equal<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::Equal(EqualsExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::And` expression.
pub fn and<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::And(AndExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::Or` expression.
pub fn or<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::Or(OrExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::Not` expression.
pub fn not<T: Into<DynProofExpr>>(expr: T) -> DynProofExpr {
    let expr = expr.into();
    DynProofExpr::Not(NotExpr::new(expr))
}

/// Creates a new `DynProofExpr::LessThan` expression.
pub fn lt<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::LessThan(InequalityExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::GreaterThan` expression.
pub fn gt<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::GreaterThan(InequalityExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::LessThanOrEqual` expression.
pub fn le<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    not(gt(lhs, rhs))
}

/// Creates a new `DynProofExpr::GreaterThanOrEqual` expression.
pub fn ge<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    not(lt(lhs, rhs))
}

/// Creates a new `DynProofExpr::Add` expression.
pub fn add<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::Add(AddSubtractExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::Subtract` expression.
pub fn subtract<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::Subtract(AddSubtractExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::Multiply` expression.
pub fn multiply<T: Into<DynProofExpr>>(lhs: T, rhs: T) -> DynProofExpr {
    let lhs = lhs.into();
    let rhs = rhs.into();
    DynProofExpr::Multiply(MultiplyExpr::new(lhs, rhs))
}

/// Creates a new `DynProofExpr::Literal` expression for a boolean value.
pub fn const_bool(val: bool) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(val))
}

/// Creates a new `DynProofExpr::Literal` expression for a smallint value.
pub fn const_smallint(val: i16) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::SmallInt(val))
}

/// Creates a new `DynProofExpr::Literal` expression for an int value.
pub fn const_int(val: i32) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int(val))
}

/// Creates a new `DynProofExpr::Literal` expression for a bigint value.
pub fn const_bigint(val: i64) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::BigInt(val))
}

/// Creates a new `DynProofExpr::Literal` expression for an int128 value.
pub fn const_int128(val: i128) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int128(val))
}

/// Creates a new `DynProofExpr::Literal` expression for a varchar value.
pub fn const_varchar(val: &str) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarChar(val.to_string()))
}

/// Creates a new `DynProofExpr::Literal` expression for a varbinary value.
pub fn const_varbinary(val: &[u8]) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarBinary(val.to_vec()))
}

/// Creates a new `DynProofExpr::Literal` expression for a scalar value.
pub fn const_scalar<S: Scalar, T: Into<S>>(val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into().into()))
}

/// Creates a new `DynProofExpr::Literal` expression for a decimal value.
pub fn const_decimal<S: Scalar, T: Into<S>>(precision: u8, scale: i8, val: T) -> DynProofExpr {
    let precision = crate::base::math::decimal::Precision::new(precision).unwrap();
    let value = crate::base::math::i256::I256::from_scalar(val.into());
    DynProofExpr::new_literal(LiteralValue::Decimal75(precision, scale, value))
}
