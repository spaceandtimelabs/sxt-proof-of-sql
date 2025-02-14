use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use core::fmt::Debug;

/// Provable AST column expression that evaluates to a `Column`
#[enum_dispatch::enum_dispatch(DynProofExpr)]
pub trait ProofExpr: Debug + Send + Sync {
    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of values. This result slice is guaranteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of values
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// [`VerificationBuilder<S>`]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError>;

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}
