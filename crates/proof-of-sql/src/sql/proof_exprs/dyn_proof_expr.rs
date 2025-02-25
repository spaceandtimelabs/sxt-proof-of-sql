use super::{
    divide_expr::DivideExpr, modulo_expr::ModuloExpr, AddSubtractExpr, AggregateExpr, AndExpr,
    ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr, MultiplyExpr, NotExpr, OrExpr, ProofExpr,
};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        parse::{type_check_binary_operation, ConversionError, ConversionResult},
        proof::{FinalRoundBuilder, VerificationBuilder},
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use core::fmt::Debug;
use proof_of_sql_parser::intermediate_ast::AggregationOperator;
use serde::{Deserialize, Serialize};
use sqlparser::ast::BinaryOperator;

/// Enum of AST column expression types that implement `ProofExpr`. Is itself a `ProofExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[enum_dispatch::enum_dispatch]
pub enum DynProofExpr {
    /// Column
    Column(ColumnExpr),
    /// Provable logical AND expression
    And(AndExpr),
    /// Provable logical OR expression
    Or(OrExpr),
    /// Provable logical NOT expression
    Not(NotExpr),
    /// Provable CONST expression
    Literal(LiteralExpr),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr),
    /// Provable numeric `+` / `-` expression
    AddSubtract(AddSubtractExpr),
    /// Provable numeric `*` expression
    Multiply(MultiplyExpr),
    /// Provable aggregate expression
    Aggregate(AggregateExpr),
    /// Provable numeric `/` expression
    Divide(DivideExpr),
    /// Provable numeric `%` expression
    Modulo(ModuloExpr),
}
impl DynProofExpr {
    /// Create column expression
    pub fn new_column(column_ref: ColumnRef) -> Self {
        Self::Column(ColumnExpr::new(column_ref))
    }
    /// Create logical AND expression
    pub fn try_new_and(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical OR expression
    pub fn try_new_or(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical NOT expression
    pub fn try_new_not(expr: DynProofExpr) -> ConversionResult<Self> {
        expr.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Not(NotExpr::new(Box::new(expr))))
    }
    /// Create CONST expression
    pub fn new_literal(value: LiteralValue) -> Self {
        Self::Literal(LiteralExpr::new(value))
    }
    /// Create a new equals expression
    pub fn try_new_equals(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Eq) {
            Ok(Self::Equals(EqualsExpr::new(Box::new(lhs), Box::new(rhs))))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }
    /// Create a new inequality expression
    pub fn try_new_inequality(
        lhs: DynProofExpr,
        rhs: DynProofExpr,
        is_lt: bool,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Lt) {
            Ok(Self::Inequality(InequalityExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                is_lt,
            )))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new add expression
    pub fn try_new_add(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Plus) {
            Ok(Self::AddSubtract(AddSubtractExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                false,
            )))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new subtract expression
    pub fn try_new_subtract(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Minus) {
            Ok(Self::AddSubtract(AddSubtractExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                true,
            )))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new multiply expression
    pub fn try_new_multiply(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Multiply) {
            Ok(Self::Multiply(MultiplyExpr::new(
                Box::new(lhs),
                Box::new(rhs),
            )))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new divide expression
    pub fn try_new_divide(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Divide) {
            Ok(Self::Divide(DivideExpr::new(Box::new(lhs), Box::new(rhs))))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new modulo expression
    pub fn try_new_modulo(lhs: DynProofExpr, rhs: DynProofExpr) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(lhs_datatype, rhs_datatype, &BinaryOperator::Modulo) {
            Ok(Self::Modulo(ModuloExpr::new(Box::new(lhs), Box::new(rhs))))
        } else {
            Err(ConversionError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new aggregate expression
    pub fn new_aggregate(op: AggregationOperator, expr: DynProofExpr) -> Self {
        Self::Aggregate(AggregateExpr::new(op, Box::new(expr)))
    }

    /// Check that the plan has the correct data type
    fn check_data_type(&self, data_type: ColumnType) -> ConversionResult<()> {
        if self.data_type() == data_type {
            Ok(())
        } else {
            Err(ConversionError::InvalidDataType {
                actual: self.data_type(),
                expected: data_type,
            })
        }
    }
}
