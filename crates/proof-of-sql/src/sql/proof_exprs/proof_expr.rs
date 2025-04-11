use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        math::decimal::Precision,
        proof::{PlaceholderResult, ProofError},
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
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of values
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// [`VerificationBuilder<S>`]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError>;

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}

/// A trait for `ProofExpr`s that always return a decimal type
pub(crate) trait DecimalProofExpr: ProofExpr {
    /// Get the precision of the expression
    ///
    /// # Panics
    /// This panics if the precision is invalid
    fn precision(&self) -> Precision {
        Precision::new(
            self.data_type()
                .precision_value()
                .expect("Precision should be valid"),
        )
        .expect("Precision should be valid")
    }

    /// Get the scale of the expression
    ///
    /// # Panics
    /// This panics if the scale is invalid
    fn scale(&self) -> i8 {
        self.data_type().scale().expect("Scale should be valid")
    }
}
