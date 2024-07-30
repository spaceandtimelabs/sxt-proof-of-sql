use super::{add_subtract_columns, scale_and_add_subtract_eval, ProvableExpr, ProvableExprPlan};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            try_add_subtract_column_types, Column, ColumnRef, ColumnType, CommitmentAccessor,
            DataAccessor,
        },
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use proof_of_sql_parser::intermediate_ast::BinaryOperator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable numerical `+` / `-` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddSubtractExpr<C: Commitment> {
    lhs: Box<ProvableExprPlan<C>>,
    rhs: Box<ProvableExprPlan<C>>,
    is_subtract: bool,
}

impl<C: Commitment> AddSubtractExpr<C> {
    /// Create numerical `+` / `-` expression
    pub fn new(
        lhs: Box<ProvableExprPlan<C>>,
        rhs: Box<ProvableExprPlan<C>>,
        is_subtract: bool,
    ) -> Self {
        Self {
            lhs,
            rhs,
            is_subtract,
        }
    }
}

impl<C: Commitment> ProvableExpr<C> for AddSubtractExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        let operator = if self.is_subtract {
            BinaryOperator::Subtract
        } else {
            BinaryOperator::Add
        };
        try_add_subtract_column_types(self.lhs.data_type(), self.rhs.data_type(), operator)
            .expect("Failed to add/subtract column types")
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> =
            self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> =
            self.rhs.result_evaluate(table_length, alloc, accessor);
        Column::Scalar(add_subtract_columns(
            lhs_column,
            rhs_column,
            self.lhs.data_type().scale().unwrap_or(0),
            self.rhs.data_type().scale().unwrap_or(0),
            alloc,
            self.is_subtract,
        ))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.add_subtract_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> = self.rhs.prover_evaluate(builder, alloc, accessor);
        Column::Scalar(add_subtract_columns(
            lhs_column,
            rhs_column,
            self.lhs.data_type().scale().unwrap_or(0),
            self.rhs.data_type().scale().unwrap_or(0),
            alloc,
            self.is_subtract,
        ))
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res =
            scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, self.is_subtract);
        Ok(res)
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
