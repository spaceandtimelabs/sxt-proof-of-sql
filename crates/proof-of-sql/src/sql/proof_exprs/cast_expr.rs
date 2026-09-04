use super::{numerical_util::unchecked_cast_column, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{
            try_cast_types, try_relabel_cast_types, Column, ColumnRef, ColumnType, LiteralValue,
            Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable CAST expression
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
}

impl CastExpr {
    /// Creates a new `CastExpr`
    pub fn try_new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> AnalyzeResult<Self> {
        let from_datatype = from_expr.data_type();
        try_cast_types(from_datatype, to_type)
            .map(|()| Self { from_expr, to_type })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_type.to_string(),
            })
    }

    /// Creates a new `CastExpr` that relabels the raw scalar encoding instead of
    /// performing a checked cast. The pair must be in the relabel whitelist
    /// ([`try_relabel_cast_types`]): anything into `Scalar`, or `Scalar` back out to
    /// a materializable type.
    ///
    /// The prover and verifier treat such a cast as a pure relabeling of the raw
    /// scalar encoding: no constraint is added and no scaling occurs. The caller must
    /// guarantee that, for every row the expression can produce, the raw scalar is a
    /// valid encoding of a `to_type` value (correct range and scale). This is only
    /// intended for composite expressions (see `super::composites`) that can make
    /// that argument structurally; misuse produces wrong results.
    pub(super) fn try_new_relabel(
        from_expr: Box<DynProofExpr>,
        to_type: ColumnType,
    ) -> AnalyzeResult<Self> {
        let from_datatype = from_expr.data_type();
        try_relabel_cast_types(from_datatype, to_type)
            .map(|()| Self { from_expr, to_type })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_type.to_string(),
            })
    }

    /// Returns the from expression
    pub fn get_from_expr(&self) -> &DynProofExpr {
        &self.from_expr
    }

    /// Returns the to type
    pub fn to_type(&self) -> &ColumnType {
        &self.to_type
    }
}

impl ProofExpr for CastExpr {
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
        Ok(unchecked_cast_column(
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
        Ok(unchecked_cast_column(
            alloc,
            uncasted_result,
            self.from_expr.data_type(),
            self.to_type,
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
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
