use crate::{
    base::{
        database::{Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor},
        scalar::ArkScalar,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
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

    /// Return the column referenced by this ColumnExpr
    pub fn get_column_reference(&self) -> ColumnRef {
        self.column_ref
    }

    /// Wrap the column output name and its type within the ColumnField
    pub fn get_column_field(&self) -> ColumnField {
        ColumnField::new(self.column_ref.column_id(), *self.column_ref.column_type())
    }

    /// Count the number of proof terms needed by this expression
    pub fn count(&self, builder: &mut CountBuilder) {
        builder.count_anchored_mles(1);
    }

    /// Evaluate the column expression and
    /// add the result to the ResultBuilder
    pub fn result_evaluate<'a>(&self, accessor: &'a dyn DataAccessor) -> Column<'a> {
        accessor.get_column(self.column_ref)
    }

    /// Given the selected rows (as a slice of booleans), evaluate the column expression and
    /// add the components needed to prove the result
    pub fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        accessor: &'a dyn DataAccessor,
    ) -> Column<'a> {
        let column = accessor.get_column(self.column_ref);
        match column {
            Column::BigInt(col) => builder.produce_anchored_mle(col),
            Column::Int128(col) => builder.produce_anchored_mle(col),
            Column::VarChar((_, scals)) => builder.produce_anchored_mle(scals),
            #[cfg(test)]
            Column::Scalar(col) => builder.produce_anchored_mle(col),
        };
        column
    }

    /// Evaluate the column expression at the sumcheck's random point,
    /// add components needed to verify this column expression
    pub fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> ArkScalar {
        let col_commit = accessor.get_commitment(self.column_ref);

        builder.consume_anchored_mle(&col_commit)
    }
}
