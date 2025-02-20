use super::{OstensibleFilterExec, filter_exec::prove_filter};
use crate::{
    base::{
        database::{
            Column, OwnedTableTestAccessor, Table, TableOptions, TableRef, TestAccessor,
            filter_util::*, owned_table_utility::*,
        },
        map::IndexMap,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProverEvaluate, ProverHonestyMarker, QueryError,
            VerifiableQueryResult,
        },
        proof_exprs::{
            ProofExpr,
            test_utility::{cols_expr_plan, column, const_int128, equal, tab},
        },
    },
    utils::log,
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExec = OstensibleFilterExec<Dishonest>;

impl ProverEvaluate for DishonestFilterExec {
    #[tracing::instrument(
        name = "DishonestFilterExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self.where_clause.result_evaluate(alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.result_evaluate(alloc, table))
            .collect();
        // Compute filtered_columns
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(output_length);

        log::log_memory_usage("End");

        res
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
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> =
            self.where_clause.prover_evaluate(builder, alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, table))
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
            table.num_rows(),
            result_len,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        res
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
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = DishonestFilterExec::new(
        cols_expr_plan(&t, &["b", "c", "d", "e"], &accessor),
        tab(&t),
        equal(column(&t, "a", &accessor), const_int128(105_i128)),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(matches!(
        res.verify(&expr, &accessor, &()),
        Err(QueryError::ProofError {
            source: ProofError::VerificationError { .. }
        })
    ));
}
