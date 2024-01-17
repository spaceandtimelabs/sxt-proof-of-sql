use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::{
        ast::BoolExpr,
        proof::{CountBuilder, ProofBuilder, VerificationBuilder},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::DynPartialEq;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable logical CONST expression
///
/// This node allows us to easily represent queries like
///    select * from T
/// and
///    select * from T where 1 = 2
/// as filter expressions with a constant where clause.
///
/// While this wouldn't be as efficient as using a new custom expression for
/// such queries, it allows us to easily support projects with minimal code
/// changes, and the performance is sufficient for present.
#[derive(Debug, DynPartialEq, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstBoolExpr {
    value: bool,
}

impl ConstBoolExpr {
    /// Create logical NOT expression
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

#[typetag::serde]
impl BoolExpr for ConstBoolExpr {
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        alloc.alloc_slice_fill_copy(table_length, self.value)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.const_bool_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        alloc.alloc_slice_fill_copy(builder.table_length(), self.value)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        _accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<ArkScalar, ProofError> {
        if self.value {
            Ok(builder.mle_evaluations.one_evaluation)
        } else {
            Ok(ArkScalar::zero())
        }
    }

    fn get_column_references(&self, _columns: &mut HashSet<ColumnRef>) {}
}
