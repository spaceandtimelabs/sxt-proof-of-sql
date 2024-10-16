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
    },
    sql::{
        parse::{type_check_binary_operation, ConversionError, ConversionResult},
        proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use sqlparser::ast::BinaryOperator;
use crate::sql::proof_exprs::aggregate_expr::AggregationOperator;

/// Enum of AST column expression types that implement `ProofExpr`. Is itself a `ProofExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DynProofExpr<C: Commitment> {
    /// Column
    Column(ColumnExpr<C>),
    /// Provable logical AND expression
    And(AndExpr<C>),
    /// Provable logical OR expression
    Or(OrExpr<C>),
    /// Provable logical NOT expression
    Not(NotExpr<C>),
    /// Provable CONST expression
    Literal(LiteralExpr<C::Scalar>),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr<C>),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr<C>),
    /// Provable numeric `+` / `-` expression
    AddSubtract(AddSubtractExpr<C>),
    /// Provable numeric `*` expression
    Multiply(MultiplyExpr<C>),
    /// Provable aggregate expression
    Aggregate(AggregateExpr<C>),
}
impl<C: Commitment> DynProofExpr<C> {
    /// Create column expression
    pub fn new_column(column_ref: ColumnRef) -> Self {
        Self::Column(ColumnExpr::new(column_ref))
    }
    /// Create logical AND expression
    pub fn try_new_and(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical OR expression
    pub fn try_new_or(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical NOT expression
    pub fn try_new_not(expr: DynProofExpr<C>) -> ConversionResult<Self> {
        expr.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Not(NotExpr::new(Box::new(expr))))
    }
    /// Create CONST expression
    pub fn new_literal(value: LiteralValue<C::Scalar>) -> Self {
        Self::Literal(LiteralExpr::new(value))
    }
    /// Create a new equals expression
    pub fn try_new_equals(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Eq ) {
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
        lhs: DynProofExpr<C>,
        rhs: DynProofExpr<C>,
        is_lte: bool,
    ) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(
            &lhs_datatype,
            &rhs_datatype,
            BinaryOperator::LtEq,
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
    pub fn try_new_add(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Plus) {
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
    pub fn try_new_subtract(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if type_check_binary_operation(&lhs_datatype, &rhs_datatype, BinaryOperator::Minus) {
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
    pub fn try_new_multiply(lhs: DynProofExpr<C>, rhs: DynProofExpr<C>) -> ConversionResult<Self> {
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
    pub fn new_aggregate(op: AggregationOperator, expr: DynProofExpr<C>) -> Self {
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

impl<C: Commitment> ProofExpr<C> for DynProofExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        match self {
            DynProofExpr::Column(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::And(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Or(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Not(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Literal(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Equals(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Inequality(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::AddSubtract(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Multiply(expr) => ProofExpr::<C>::count(expr, builder),
            DynProofExpr::Aggregate(expr) => ProofExpr::<C>::count(expr, builder),
        }
    }

    fn data_type(&self) -> ColumnType {
        match self {
            DynProofExpr::Column(expr) => expr.data_type(),
            DynProofExpr::AddSubtract(expr) => expr.data_type(),
            DynProofExpr::Multiply(expr) => expr.data_type(),
            DynProofExpr::Aggregate(expr) => expr.data_type(),
            DynProofExpr::Literal(expr) => ProofExpr::<C>::data_type(expr),
            DynProofExpr::And(_)
            | DynProofExpr::Or(_)
            | DynProofExpr::Not(_)
            | DynProofExpr::Equals(_)
            | DynProofExpr::Inequality(_) => ColumnType::Boolean,
        }
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self {
            DynProofExpr::Column(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::And(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Or(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Not(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Literal(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Equals(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Inequality(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::AddSubtract(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Multiply(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
            DynProofExpr::Aggregate(expr) => {
                ProofExpr::<C>::result_evaluate(expr, table_length, alloc, accessor)
            }
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        match self {
            DynProofExpr::Column(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::And(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Or(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Not(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Literal(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Equals(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Inequality(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::AddSubtract(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Multiply(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
            DynProofExpr::Aggregate(expr) => {
                ProofExpr::<C>::prover_evaluate(expr, builder, alloc, accessor)
            }
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        match self {
            DynProofExpr::Column(expr) => {
                ProofExpr::<C>::verifier_evaluate(expr, builder, accessor)
            }
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
            DynProofExpr::Column(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::And(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Or(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Not(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Literal(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Equals(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Inequality(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::AddSubtract(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Multiply(expr) => ProofExpr::<C>::get_column_references(expr, columns),
            DynProofExpr::Aggregate(expr) => ProofExpr::<C>::get_column_references(expr, columns),
        }
    }
}
