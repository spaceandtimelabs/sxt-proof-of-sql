use super::{add_subtract_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{
            try_add_subtract_column_types_of_same_decimal_type, Column, ColumnOperationResult,
            ColumnRef, ColumnType, LiteralValue, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable numerical `+` / `-` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddSubtractExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
    is_subtract: bool,
}

impl AddSubtractExpr {
    /// Create numerical `+` / `-` expression
    pub fn try_new(
        lhs: Box<DynProofExpr>,
        rhs: Box<DynProofExpr>,
        is_subtract: bool,
    ) -> ColumnOperationResult<Self> {
        try_add_subtract_column_types_of_same_decimal_type(lhs.data_type(), rhs.data_type())?;
        Ok(Self {
            lhs,
            rhs,
            is_subtract,
        })
    }
}

impl ProofExpr for AddSubtractExpr {
    fn data_type(&self) -> ColumnType {
        self.lhs.data_type()
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        Ok(Column::Scalar(add_subtract_columns(
            lhs_column,
            rhs_column,
            alloc,
            self.is_subtract,
        )))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.add_subtract_expr.final_round_evaluate",
        level = "info",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let res = Column::Scalar(add_subtract_columns(
            lhs_column,
            rhs_column,
            alloc,
            self.is_subtract,
        ));

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        Ok(if self.is_subtract {
            lhs_eval - rhs_eval
        } else {
            lhs_eval + rhs_eval
        })
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
