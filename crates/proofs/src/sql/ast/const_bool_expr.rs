use crate::base::database::{ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::polynomial::ArkScalar;
use crate::sql::ast::BoolExpr;
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};
use num_traits::Zero;

use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
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
#[derive(Debug, DynPartialEq, PartialEq, Eq)]
pub struct ConstBoolExpr {
    value: bool,
}

impl ConstBoolExpr {
    /// Create logical NOT expression
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl BoolExpr for ConstBoolExpr {
    fn count(&self, _counts: &mut ProofCounts) {}

    #[tracing::instrument(
        name = "proofs.sql.ast.const_bool_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        _builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        alloc.alloc_slice_fill_copy(counts.table_length, self.value)
    }

    fn verifier_evaluate_ark(
        &self,
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) -> ArkScalar {
        if self.value {
            builder.mle_evaluations.get_one_evaluation_ark()
        } else {
            ArkScalar::zero()
        }
    }

    fn get_column_references(&self, _columns: &mut HashSet<ColumnRef>) {}
}
