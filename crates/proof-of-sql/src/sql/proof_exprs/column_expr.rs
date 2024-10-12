use super::ProofExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use core::marker::PhantomData;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};
/// Provable expression for a column
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ColumnExpr<C: Commitment> {
    column_ref: ColumnRef,
    _phantom_data: PhantomData<C>,
}

impl<C: Commitment> ColumnExpr<C> {
    /// Create a new column expression
    pub fn new(column_ref: ColumnRef) -> Self {
        Self {
            column_ref,
            _phantom_data: PhantomData,
        }
    }

    /// Return the column referenced by this [`ColumnExpr`]
    pub fn get_column_reference(&self) -> ColumnRef {
        self.column_ref
    }

    /// Wrap the column output name and its type within the [`ColumnField`]
    pub fn get_column_field(&self) -> ColumnField {
        ColumnField::new(self.column_ref.column_id(), *self.column_ref.column_type())
    }

    /// Get the column identifier
    pub fn column_id(&self) -> Identifier {
        self.column_ref.column_id()
    }
}

impl<C: Commitment> ProofExpr<C> for ColumnExpr<C> {
    /// Count the number of proof terms needed by this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        builder.count_anchored_mles(1);
        Ok(())
    }

    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType {
        *self.get_column_reference().column_type()
    }

    /// Evaluate the column expression and
    /// add the result to the [`ResultBuilder`](crate::sql::proof::ResultBuilder)
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let column = accessor.get_column(self.column_ref);
        assert_eq!(column.len(), table_length);
        column
    }

    /// Given the selected rows (as a slice of booleans), evaluate the column expression and
    /// add the components needed to prove the result
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let column = accessor.get_column(self.column_ref);
        builder.produce_anchored_mle(column);
        column
    }

    /// Evaluate the column expression at the sumcheck's random point,
    /// add components needed to verify this column expression
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let col_commit = accessor.get_commitment(self.column_ref);
        Ok(builder.consume_anchored_mle(col_commit))
    }

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}
