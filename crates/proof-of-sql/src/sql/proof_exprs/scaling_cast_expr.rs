use super::{
    numerical_util::{cast_column_with_scaling, try_get_scaling_factor_with_precision_and_scale},
    DynProofExpr, ProofExpr,
};
use crate::{
    base::{
        database::{Column, ColumnOperationResult, ColumnRef, ColumnType, LiteralValue, Table},
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
pub struct ScalingCastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
    scaling_factor: [u64; 4],
}

impl ScalingCastExpr {
    /// Creates a new `CastExpr`
    pub fn try_new(
        from_expr: Box<DynProofExpr>,
        to_type: ColumnType,
    ) -> ColumnOperationResult<Self> {
        let scaling_factor =
            try_get_scaling_factor_with_precision_and_scale(from_expr.data_type(), to_type)?.0;
        Ok(Self {
            from_expr,
            to_type,
            scaling_factor: scaling_factor.into(),
        })
    }
}

impl ProofExpr for ScalingCastExpr {
    fn data_type(&self) -> ColumnType {
        self.to_type
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self.from_expr.first_round_evaluate(alloc, table, params)?;
        Ok(cast_column_with_scaling(
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
        Ok(cast_column_with_scaling(
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
        self.from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)
            .map(|unscaled_eval| S::from(self.scaling_factor) * unscaled_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
