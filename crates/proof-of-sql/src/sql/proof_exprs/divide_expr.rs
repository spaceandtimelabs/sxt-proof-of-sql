use super::{numerical_util::divide_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        proof_gadgets::divide_and_modulo_expr::DivideAndModuloExpr,
    },
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable numerical `/` expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivideExpr {
    inner_expr: DivideAndModuloExpr,
}

impl DivideExpr {
    /// Create numerical `/` expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self {
            inner_expr: DivideAndModuloExpr::new(lhs, rhs),
        }
    }
}

impl ProofExpr for DivideExpr {
    fn data_type(&self) -> ColumnType {
        self.inner_expr.data_type()
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.inner_expr.lhs.result_evaluate(alloc, table);
        let rhs_column: Column<'a, S> = self.inner_expr.rhs.result_evaluate(alloc, table);
        divide_columns(&lhs_column, &rhs_column, alloc).0
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        self.inner_expr.prover_evaluate(builder, alloc, table).0
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<S, ProofError> {
        Ok(self
            .inner_expr
            .verifier_evaluate(builder, accessor, one_eval)?
            .0)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.inner_expr.get_column_references(columns);
    }
}
