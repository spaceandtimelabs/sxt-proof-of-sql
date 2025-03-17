use super::error::Error;
use crate::{
    base::{
        database::{ColumnRef, LiteralValue},
        map::IndexSet,
    },
    sql::proof_exprs::{self, DynProofExpr},
};
use alloc::boxed::Box;
use serde::Serialize;

/// Represents an expression that can be serialized for EVM.
#[derive(Serialize)]
pub enum Expr {
    Column(ColumnExpr),
    Equals(EqualsExpr),
    Literal(LiteralExpr),
}
impl Expr {
    /// Try to create an `Expr` from a `DynProofExpr`.
    pub fn try_from_proof_expr(
        expr: &DynProofExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<Self, Error> {
        match expr {
            DynProofExpr::Column(column_expr) => {
                ColumnExpr::try_from_proof_expr(column_expr, column_refs).map(Self::Column)
            }
            DynProofExpr::Literal(literal_expr) => {
                LiteralExpr::try_from_proof_expr(literal_expr).map(Self::Literal)
            }
            DynProofExpr::Equals(equals_expr) => {
                EqualsExpr::try_from_proof_expr(equals_expr, column_refs).map(Self::Equals)
            }
            _ => Err(Error::NotSupported),
        }
    }
}

/// Represents a column expression.
#[derive(Serialize)]
pub struct ColumnExpr {
    column_number: usize,
}
impl ColumnExpr {
    /// Try to create a `ColumnExpr` from a `proof_exprs::ColumnExpr`.
    fn try_from_proof_expr(
        expr: &proof_exprs::ColumnExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<Self, Error> {
        Ok(Self {
            column_number: column_refs
                .get_index_of(&expr.column_ref)
                .ok_or(Error::ColumnNotFound)?,
        })
    }
}

/// Represents a literal expression.
#[derive(Serialize)]
pub enum LiteralExpr {
    BigInt(i64),
}
impl LiteralExpr {
    /// Try to create a `LiteralExpr` from a `proof_exprs::LiteralExpr`.
    fn try_from_proof_expr(expr: &proof_exprs::LiteralExpr) -> Result<Self, Error> {
        match expr.value {
            LiteralValue::BigInt(value) => Ok(LiteralExpr::BigInt(value)),
            _ => Err(Error::NotSupported),
        }
    }
}

/// Represents an equals expression.
#[derive(Serialize)]
pub struct EqualsExpr {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}
impl EqualsExpr {
    /// Try to create an `EqualsExpr` from a `proof_exprs::EqualsExpr`.
    fn try_from_proof_expr(
        expr: &proof_exprs::EqualsExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<Self, Error> {
        Ok(EqualsExpr {
            lhs: Box::new(Expr::try_from_proof_expr(&expr.lhs, column_refs)?),
            rhs: Box::new(Expr::try_from_proof_expr(&expr.rhs, column_refs)?),
        })
    }
}
