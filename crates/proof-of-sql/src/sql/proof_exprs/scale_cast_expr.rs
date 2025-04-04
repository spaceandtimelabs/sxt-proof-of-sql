use super::{
    scale_cast_column, try_get_scaling_factor_with_precision_and_scale, DynProofExpr, ProofExpr,
};
use crate::{
    base::{
        database::{try_scale_cast_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable CAST expression
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ScaleCastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
}

impl ScaleCastExpr {
    /// Creates a new `ScaleCastExpr`
    pub fn new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> Self {
        Self { from_expr, to_type }
    }
}

impl ProofExpr for ScaleCastExpr {
    fn data_type(&self) -> ColumnType {
        try_scale_cast_types(self.from_expr.data_type(), self.to_type)
            .expect("Failed to cast column type");
        self.to_type
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self.from_expr.first_round_evaluate(alloc, table, params)?;
        Ok(scale_cast_column(
            alloc,
            uncasted_result,
            self.from_expr.data_type(),
            self.to_type,
        ))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self
            .from_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        Ok(scale_cast_column(
            alloc,
            uncasted_result,
            self.from_expr.data_type(),
            self.to_type,
        ))
    }

    /// # Panics
    ///
    /// Panics if any of the following is true
    /// 1. `from_expr` or `to_type` is not numeric
    /// 2. scale decreases when the casting takes place
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let input_eval = self
            .from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let scaling_factor = try_get_scaling_factor_with_precision_and_scale(
            self.from_expr.data_type(),
            self.to_type,
        ).expect("ScaleCastExpr does not work on non-numeric types").0;
        Ok(input_eval * scaling_factor)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
