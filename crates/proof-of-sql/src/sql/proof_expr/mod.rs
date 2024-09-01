//! This module proves provable expressions.
mod aliased_dyn_proof_expr;
pub(crate) use aliased_dyn_proof_expr::AliasedDynProofExpr;

mod add_subtract_expr;
pub(crate) use add_subtract_expr::AddSubtractExpr;
#[cfg(all(test, feature = "blitzar"))]
mod add_subtract_expr_test;

mod aggregate_expr;
pub(crate) use aggregate_expr::AggregateExpr;

mod multiply_expr;
use multiply_expr::MultiplyExpr;
#[cfg(all(test, feature = "blitzar"))]
mod multiply_expr_test;

mod bitwise_verification;
use bitwise_verification::*;
#[cfg(test)]
mod bitwise_verification_test;

mod dyn_proof_expr;
pub(crate) use dyn_proof_expr::DynProofExpr;

#[cfg(all(test, feature = "blitzar"))]
mod proof_expr_test;

mod literal_expr;
pub(crate) use literal_expr::LiteralExpr;
#[cfg(all(test, feature = "blitzar"))]
mod literal_expr_test;

mod and_expr;
use and_expr::AndExpr;
#[cfg(all(test, feature = "blitzar"))]
mod and_expr_test;

mod inequality_expr;
use inequality_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod inequality_expr_test;

mod or_expr;
use or_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod or_expr_test;

mod not_expr;
use not_expr::NotExpr;
#[cfg(all(test, feature = "blitzar"))]
mod not_expr_test;

mod comparison_util;
pub(crate) use comparison_util::scale_and_subtract;

mod numerical_util;
pub(crate) use numerical_util::{
    add_subtract_columns, multiply_columns, scale_and_add_subtract_eval,
};

mod equals_expr;
use equals_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod equals_expr_test;

mod sign_expr;
use sign_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;

mod table_expr;
pub(crate) use table_expr::TableExpr;

#[cfg(test)]
pub(crate) mod test_utility;

mod column_expr;
pub(crate) use column_expr::ColumnExpr;
#[cfg(all(test, feature = "blitzar"))]
mod column_expr_test;

use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use indexmap::IndexSet;
use std::fmt::Debug;

/// Provable AST column expression that evaluates to a `Column`
pub trait ProofExpr<C: Commitment>: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError>;

    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of values. This result slice is guarenteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of values
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError>;

    /// Insert in the IndexSet `columns` all the column
    /// references in the BoolExpr or forwards the call to some
    /// subsequent bool_expr
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}
