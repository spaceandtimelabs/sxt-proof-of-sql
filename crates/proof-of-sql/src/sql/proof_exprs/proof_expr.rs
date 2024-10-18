use crate::base::database::ColumnTypeKind;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use core::fmt::Debug;

/// Provable AST column expression that evaluates to a `Column`
pub trait ProofExpr<C: Commitment>: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError>;

    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;
    /// Get the data type of the expression
    fn data_type_kind(&self) -> ColumnTypeKind {
        self.data_type().get_kind()
    }

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
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// [`VerificationBuilder`]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError>;

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}
