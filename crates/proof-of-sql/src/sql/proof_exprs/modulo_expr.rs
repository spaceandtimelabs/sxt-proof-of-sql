use super::{numerical_util::modulo_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_divide_modulo_column_types, Column, ColumnRef, Table},
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
pub struct ModuloExpr {
    inner_expr: DivideAndModuloExpr,
}

impl ModuloExpr {
    /// Create numerical `%` expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self {
            inner_expr: DivideAndModuloExpr::new(lhs, rhs),
        }
    }
}

impl ProofExpr for ModuloExpr {
    fn data_type(&self) -> crate::base::database::ColumnType {
        try_divide_modulo_column_types(
            self.inner_expr.lhs.data_type(),
            self.inner_expr.rhs.data_type(),
        )
        .expect("Failed to take modulo of column types")
        .1
    }

    fn result_evaluate<'a, S: crate::base::scalar::Scalar>(
        &self,
        alloc: &'a bumpalo::Bump,
        table: &crate::base::database::Table<'a, S>,
    ) -> crate::base::database::Column<'a, S> {
        let lhs_column: Column<'a, S> = self.inner_expr.lhs.result_evaluate(alloc, table);
        let rhs_column: Column<'a, S> = self.inner_expr.rhs.result_evaluate(alloc, table);
        modulo_columns(&lhs_column, &rhs_column, alloc)
    }

    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        self.inner_expr.prover_evaluate(builder, alloc, table).1
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
            .1)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.inner_expr.get_column_references(columns);
    }
}
