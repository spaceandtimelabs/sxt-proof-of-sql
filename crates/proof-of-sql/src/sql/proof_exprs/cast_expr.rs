use super::{numerical_util::cast_column, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_cast_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        PlaceholderProverResult,
    },
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable CAST expression
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
}

impl CastExpr {
    /// Creates a new `CastExpr`
    pub fn new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> Self {
        Self { from_expr, to_type }
    }
}

impl ProofExpr for CastExpr {
    fn data_type(&self) -> ColumnType {
        try_cast_types(self.from_expr.data_type(), self.to_type)
            .expect("Failed to cast column type");
        self.to_type
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderProverResult<Column<'a, S>> {
        let uncasted_result = self.from_expr.first_round_evaluate(alloc, table, params)?;
        Ok(cast_column(alloc, uncasted_result, self.to_type))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderProverResult<Column<'a, S>> {
        let uncasted_result = self
            .from_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        Ok(cast_column(alloc, uncasted_result, self.to_type))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        self.from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
