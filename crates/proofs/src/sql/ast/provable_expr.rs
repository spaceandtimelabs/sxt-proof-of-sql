use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use std::{collections::HashSet, fmt::Debug};

/// Provable AST column expression that evaluates to a T
pub trait ProvableExpr<C: Commitment, T>: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError>;

    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of T values. This result slice is guarenteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [T];

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of T values
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [T];

    /// Compute the evaluation of a multilinear extension from this T expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError>;

    /// Insert in the HashSet `columns` all the column
    /// references in the BoolExpr or forwards the call to some
    /// subsequent bool_expr
    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>);
}
