use super::{
    AddSubtractExpr, AggregateExpr, AndExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr,
    MultiplyExpr, NotExpr, OrExpr, ProofExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, LiteralValue},
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        parse::{type_check_binary_operation, ConversionError, ConversionResult},
        proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use core::fmt::Debug;
use proof_of_sql_parser::intermediate_ast::{AggregationOperator, BinaryOperator};
use serde::{Deserialize, Serialize};

/// Enum of AST column expression types that implement `ProofExpr`. Is itself a `ProofExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Equal) {
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
        is_lte: bool,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(
            &lhs_datatype,
            &rhs_datatype,
            BinaryOperator::LessThanOrEqual,
        ) {
            Ok(Self::Inequality(InequalityExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                is_lte,
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
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Add) {
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
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Subtract) {
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
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Multiply) {
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

impl ProofExpr for DynProofExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            DynProofExpr::Column(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::And(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Or(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Not(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Literal(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Equals(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Inequality(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::AddSubtract(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Multiply(expr) => ProofExpr::count(expr, builder),
            DynProofExpr::Aggregate(expr) => ProofExpr::count(expr, builder),
        }
    }

    fn data_type(&self) -> ColumnType {
        match self {
            DynProofExpr::Column(expr) => expr.data_type(),
            DynProofExpr::AddSubtract(expr) => expr.data_type(),
            DynProofExpr::Multiply(expr) => expr.data_type(),
            DynProofExpr::Aggregate(expr) => expr.data_type(),
            DynProofExpr::Literal(expr) => ProofExpr::data_type(expr),
            DynProofExpr::And(_)
            | DynProofExpr::Or(_)
            | DynProofExpr::Not(_)
            | DynProofExpr::Equals(_)
            | DynProofExpr::Inequality(_) => ColumnType::Boolean,
        }
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        match self {
            DynProofExpr::Column(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::And(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Or(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Not(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Literal(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Equals(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Inequality(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::AddSubtract(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Multiply(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Aggregate(expr) => {
                ProofExpr::result_evaluate(expr, table_length, alloc, accessor)
            }
        }
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        match self {
            DynProofExpr::Column(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::And(expr) => ProofExpr::prover_evaluate(expr, builder, alloc, accessor),
            DynProofExpr::Or(expr) => ProofExpr::prover_evaluate(expr, builder, alloc, accessor),
            DynProofExpr::Not(expr) => ProofExpr::prover_evaluate(expr, builder, alloc, accessor),
            DynProofExpr::Literal(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Equals(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Inequality(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::AddSubtract(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Multiply(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Aggregate(expr) => {
                ProofExpr::prover_evaluate(expr, builder, alloc, accessor)
            }
        }
    }

    fn verifier_evaluate<C: Commitment>(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        match self {
            DynProofExpr::Column(expr) => ProofExpr::verifier_evaluate(expr, builder, accessor),
            DynProofExpr::And(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Or(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Not(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Literal(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Equals(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Inequality(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::AddSubtract(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Multiply(expr) => expr.verifier_evaluate(builder, accessor),
            DynProofExpr::Aggregate(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        match self {
            DynProofExpr::Column(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::And(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Or(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Not(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Literal(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Equals(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Inequality(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::AddSubtract(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Multiply(expr) => ProofExpr::get_column_references(expr, columns),
            DynProofExpr::Aggregate(expr) => ProofExpr::get_column_references(expr, columns),
        }
    }
}
