use super::{add_subtract_columns, scale_and_add_subtract_eval, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_add_subtract_column_types, Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
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
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>, is_subtract: bool) -> Self {
        Self {
            lhs,
            rhs,
            is_subtract,
        }
    }
}

impl ProofExpr for AddSubtractExpr {
    fn data_type(&self) -> ColumnType {
        try_add_subtract_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to add/subtract column types")
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.result_evaluate(alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.result_evaluate(alloc, table);
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
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.prover_evaluate(builder, alloc, table);
        Column::Scalar(add_subtract_columns(
            lhs_column,
            rhs_column,
            self.lhs.data_type().scale().unwrap_or(0),
            self.rhs.data_type().scale().unwrap_or(0),
            alloc,
            self.is_subtract,
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<S, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor, one_eval)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor, one_eval)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res =
            scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, self.is_subtract);
        Ok(res)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
