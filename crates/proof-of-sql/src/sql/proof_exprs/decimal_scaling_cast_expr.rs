use super::{
    numerical_util::{
        cast_column_to_decimal_with_scaling, try_get_scaling_factor_with_precision_and_scale,
    },
    DynProofExpr, ProofExpr,
};
use crate::{
    base::{
        database::{
            try_decimal_scale_cast_types, Column, ColumnRef, ColumnType, LiteralValue, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DecimalScalingCastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
}

impl DecimalScalingCastExpr {
    /// Creates a new `CastExpr`
    pub fn new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> Self {
        Self { from_expr, to_type }
    }
}

impl ProofExpr for DecimalScalingCastExpr {
    fn data_type(&self) -> ColumnType {
        try_decimal_scale_cast_types(self.from_expr.data_type(), self.to_type)
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
        Ok(cast_column_to_decimal_with_scaling(
            alloc,
            uncasted_result,
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
        Ok(cast_column_to_decimal_with_scaling(
            alloc,
            uncasted_result,
            self.to_type,
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let unscaled_eval = self
            .from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        try_get_scaling_factor_with_precision_and_scale::<S>(
            self.from_expr.data_type(),
            self.to_type,
        )
        .map(|(scaling_factor, _, _)| scaling_factor * unscaled_eval)
        .map_err(|_| ProofError::UnsupportedQueryPlan {
            error: "Invalid scale cast",
        })
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
