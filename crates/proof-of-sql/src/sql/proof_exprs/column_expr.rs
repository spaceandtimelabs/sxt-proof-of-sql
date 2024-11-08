use super::ProofExpr;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnField, ColumnRef, ColumnType, DataAccessor},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};
/// Provable expression for a column
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ColumnExpr {
    column_ref: ColumnRef,
}

impl ColumnExpr {
    /// Create a new column expression
    pub fn new(column_ref: ColumnRef) -> Self {
        Self { column_ref }
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

impl ProofExpr for ColumnExpr {
    /// Count the number of proof terms needed by this expression
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        Ok(())
    }

    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType {
        *self.get_column_reference().column_type()
    }

    /// Evaluate the column expression and
    /// add the result to the [`FirstRoundBuilder`](crate::sql::proof::FirstRoundBuilder)
    fn result_evaluate<'a, S: Scalar>(
        &self,
        table_length: usize,
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        let column = accessor.get_column(self.column_ref);
        assert_eq!(column.len(), table_length);
        column
    }

    /// Given the selected rows (as a slice of booleans), evaluate the column expression and
    /// add the components needed to prove the result
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        accessor.get_column(self.column_ref)
    }

    /// Evaluate the column expression at the sumcheck's random point,
    /// add components needed to verify this column expression
    fn verifier_evaluate<C: Commitment>(
        &self,
        _builder: &mut VerificationBuilder<C>,
        accessor: &IndexMap<ColumnRef, C::Scalar>,
    ) -> Result<C::Scalar, ProofError> {
        Ok(*accessor
            .get(&self.column_ref)
            .ok_or(ProofError::VerificationError {
                error: "Column Not Found",
            })?)
    }

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}
