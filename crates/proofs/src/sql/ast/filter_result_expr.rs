use crate::base::database::{Column, ColumnRef, CommitmentAccessor, DataAccessor};
use crate::sql::proof::{
    make_sumcheck_term, DenseProvableResultColumn, ProofBuilder, ProofCounts,
    SumcheckSubpolynomial, VerificationBuilder,
};
use arrow::datatypes::Field;
use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use proofs_sql::Identifier;
use std::cmp::max;
use std::collections::HashSet;

/// Provable expression for a result column within a filter SQL expression
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FilterResultExpr {
    column_ref: ColumnRef,
    column_name_id: Identifier,
}

impl FilterResultExpr {
    /// Create a new filter result expression
    pub fn new(column_ref: ColumnRef, column_name_id: Identifier) -> Self {
        Self {
            column_ref,
            column_name_id,
        }
    }

    /// Add the `self.column_ref` to the `columns` HashSet
    pub fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }

    /// Wrap the column output name and its type within the arrow Field
    pub fn get_field(&self) -> Field {
        Field::new(
            self.column_name_id.name(),
            self.column_ref.column_type().into(),
            false,
        )
    }

    /// Count the number of proof terms needed by this expression
    pub fn count(&self, counts: &mut ProofCounts) {
        counts.result_columns += 1;
        counts.sumcheck_subpolynomials += 1;
        counts.anchored_mles += 1;
        counts.sumcheck_max_multiplicands = max(counts.sumcheck_max_multiplicands, 3);
    }

    /// Given the selected rows (as a slice of booleans), evaluate the filter result expression and
    /// add the components needed to prove the result
    pub fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
        selection: &'a [bool],
    ) {
        let Column::BigInt(col) = accessor.get_column(self.column_ref);

        // add result column
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));

        // add MLE for result column
        builder.produce_anchored_mle(col);

        // make a column of selected result values only
        let selected_vals =
            alloc.alloc_slice_fill_with(
                counts.table_length,
                |i| if selection[i] { col[i] } else { 0 },
            );

        // add sumcheck term for col * selection
        let terms = vec![
            (
                Scalar::one(),
                vec![make_sumcheck_term(counts.sumcheck_variables, selected_vals)],
            ),
            (
                -Scalar::one(),
                vec![
                    make_sumcheck_term(counts.sumcheck_variables, col),
                    make_sumcheck_term(counts.sumcheck_variables, selection),
                ],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }

    /// Given the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this filter result expression
    pub fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
        selection_eval: &Scalar,
    ) {
        let col_commit = accessor.get_commitment(self.column_ref);

        let result_eval = builder.consume_result_mle();
        let col_eval = builder.consume_anchored_mle(&col_commit);

        let poly_eval =
            builder.mle_evaluations.random_evaluation * (result_eval - col_eval * selection_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&poly_eval);
    }
}
