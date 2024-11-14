use super::{filter_exec::prove_filter, OstensibleFilterExec};
use crate::{
    base::{
        database::{
            filter_util::*, owned_table_utility::*, Column, DataAccessor, OwnedTableTestAccessor,
            Table, TableOptions, TestAccessor,
        },
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, ProverHonestyMarker,
            QueryError, VerifiableQueryResult,
        },
        proof_exprs::{
            test_utility::{cols_expr_plan, column, const_int128, equal, tab},
            ProofExpr,
        },
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExec = OstensibleFilterExec<Dishonest>;

impl ProverEvaluate for DishonestFilterExec {
    #[tracing::instrument(
        name = "DishonestFilterExec::result_evaluate",
        level = "debug",
        skip_all
    )]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        let column_refs = self.get_column_references();
        let used_table = accessor.get_table(self.table.table_ref, &column_refs);
        // 1. selection
        let selection_column: Column<'a, S> = self.where_clause.result_evaluate(alloc, &used_table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.result_evaluate(alloc, &used_table))
            .collect();
        // Compute filtered_columns
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias)
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator")
    }

    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(
        name = "DishonestFilterExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        let column_refs = self.get_column_references();
        let used_table = accessor.get_table(self.table.table_ref, &column_refs);
        // 1. selection
        let selection_column: Column<'a, S> =
            self.where_clause
                .prover_evaluate(builder, alloc, &used_table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .prover_evaluate(builder, alloc, &used_table)
            })
            .collect();
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            selection,
            &filtered_columns,
            result_len,
        );
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias)
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator")
    }
}

/// Tamper with the first element of the first column that is a Scalar. This could be changed for different types of tests.
fn tamper_column<'a, S: Scalar>(
    alloc: &'a Bump,
    mut columns: Vec<Column<'a, S>>,
) -> Vec<Column<'a, S>> {
    for column in &mut columns {
        if let Column::Scalar(tampered_column) = column {
            if !tampered_column.is_empty() {
                let tampered_column = alloc.alloc_slice_copy(tampered_column);
                // The following could be changed for different types of tests, but for the simplest one, we will simply increase the first element by 1.
                tampered_column[0] += S::one();
                *column = Column::Scalar(tampered_column);
                break;
            }
        }
    }
    columns
}

#[test]
fn we_fail_to_verify_a_basic_filter_with_a_dishonest_prover() {
    let data = owned_table([
        bigint("a", [101, 104, 105, 102, 105]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = DishonestFilterExec::new(
        cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
        tab(t),
        equal(column(t, "a", &accessor), const_int128(105_i128)),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(matches!(
        res.verify(&expr, &accessor, &()),
        Err(QueryError::ProofError {
            source: ProofError::VerificationError { .. }
        })
    ));
}
