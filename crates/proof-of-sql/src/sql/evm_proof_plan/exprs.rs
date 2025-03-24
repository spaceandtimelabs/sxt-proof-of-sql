use super::error::Error;
use crate::{
    base::{
        database::{ColumnRef, LiteralValue},
        map::IndexSet,
    },
    sql::proof_exprs::{self, DynProofExpr},
};
use alloc::boxed::Box;
use serde::{Deserialize, Serialize};

/// Represents an expression that can be serialized for EVM.
#[derive(Serialize, Deserialize)]
pub(super) enum Expr {
    Column(ColumnExpr),
    Equals(EqualsExpr),
    Literal(LiteralExpr),
}
impl Expr {
    /// Try to create an `Expr` from a `DynProofExpr`.
    pub(super) fn try_from_proof_expr(
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

    pub(super) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<DynProofExpr, Error> {
        match self {
            Expr::Column(column_expr) => Ok(DynProofExpr::Column(
                column_expr.try_into_proof_expr(column_refs)?,
            )),
            Expr::Equals(equals_expr) => Ok(DynProofExpr::Equals(
                equals_expr.try_into_proof_expr(column_refs)?,
            )),
            Expr::Literal(literal_expr) => Ok(DynProofExpr::Literal(literal_expr.to_proof_expr())),
        }
    }
}

/// Represents a column expression.
#[derive(Serialize, Deserialize)]
pub(super) struct ColumnExpr {
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

    fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<proof_exprs::ColumnExpr, Error> {
        Ok(proof_exprs::ColumnExpr::new(
            column_refs
                .get_index(self.column_number)
                .ok_or(Error::ColumnNotFound)?
                .clone(),
        ))
    }
}

/// Represents a literal expression.
#[derive(Serialize, Deserialize)]
pub(super) enum LiteralExpr {
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

    fn to_proof_expr(&self) -> proof_exprs::LiteralExpr {
        match self {
            LiteralExpr::BigInt(value) => {
                proof_exprs::LiteralExpr::new(LiteralValue::BigInt(*value))
            }
        }
    }
}

/// Represents an equals expression.
#[derive(Serialize, Deserialize)]
pub(super) struct EqualsExpr {
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

    fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<proof_exprs::EqualsExpr, Error> {
        Ok(proof_exprs::EqualsExpr {
            lhs: Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            rhs: Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        })
    }
}
