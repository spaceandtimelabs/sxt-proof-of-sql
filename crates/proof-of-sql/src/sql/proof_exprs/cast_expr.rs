use super::{numerical_util::cast_column, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_cast_types, Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
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
    #[expect(dead_code)]
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

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let uncasted_result = self.from_expr.result_evaluate(alloc, table);
        cast_column(alloc, uncasted_result, self.to_type)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let uncasted_result = self.from_expr.prover_evaluate(builder, alloc, table);
        cast_column(alloc, uncasted_result, self.to_type)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        self.from_expr
            .verifier_evaluate(builder, accessor, chi_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
