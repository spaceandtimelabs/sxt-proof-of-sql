use super::{scale_and_add_subtract_eval, scale_and_subtract, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        proof_gadgets::{
            final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign,
        },
    },
    utils::log,
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable AST expression for an inequality expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
    is_lt: bool,
}

impl InequalityExpr {
    /// Create a new less than or equal
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>, is_lt: bool) -> Self {
        Self { lhs, rhs, is_lt }
    }
}

impl ProofExpr for InequalityExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(
        name = "InequalityExpr::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column = self.rhs.first_round_evaluate(alloc, table, params)?;
        let table_length = table.num_rows();
        let diff = if self.is_lt {
            scale_and_subtract(alloc, lhs_column, rhs_column, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, false)
                .expect("Failed to scale and subtract")
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(first_round_evaluate_sign(table_length, alloc, diff));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "InequalityExpr::final_round_evaluate",
        level = "debug",
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

        let lhs_column = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let diff = if self.is_lt {
            scale_and_subtract(alloc, lhs_column, rhs_column, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, false)
                .expect("Failed to scale and subtract")
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(final_round_evaluate_sign(builder, alloc, diff));

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
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff_eval = if self.is_lt {
            scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, true)
        } else {
            scale_and_add_subtract_eval(rhs_eval, lhs_eval, rhs_scale, lhs_scale, true)
        };

        // sign(diff) == -1
        verifier_evaluate_sign(builder, diff_eval, chi_eval, None)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
