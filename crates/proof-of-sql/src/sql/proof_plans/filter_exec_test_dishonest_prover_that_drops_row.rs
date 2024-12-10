use super::{fold_columns, OstensibleFilterExec};
use crate::{
    base::{
        database::{
            filter_util::filter_columns, owned_table_utility::*, Column, OwnedTableTestAccessor,
            Table, TableOptions, TableRef, TestAccessor,
        },
        map::IndexMap,
        scalar::Scalar,
        slice_ops,
    },
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProverEvaluate, ProverHonestyMarker,
            SumcheckSubpolynomialType, VerifiableQueryResult,
        },
        proof_exprs::{
            test_utility::{cols_expr_plan, column, const_int128, equal, tab},
            ProofExpr,
        },
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use ark_std::test_rng;
use bumpalo::Bump;
use num_traits::{One, Zero};

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExec = OstensibleFilterExec<Dishonest>;

impl ProverEvaluate for DishonestFilterExec {
    #[tracing::instrument(name = "FilterExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self.where_clause.result_evaluate(alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count() - 1;

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.result_evaluate(alloc, table))
            .collect();

        // Compute filtered_columns and indexes
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|expr| expr.alias).zip(
                filtered_columns
                    .into_iter()
                    .map(|col| col.slice_range_from(1..)),
            ),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        builder.produce_one_evaluation_length(output_length);
        res
    }

    #[tracing::instrument(name = "FilterExec::final_round_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> =
            self.where_clause.prover_evaluate(builder, alloc, table);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count() - 1;

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, table))
            .collect();
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column.slice_range_from(1..));
        });

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter::<S>(
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
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|expr| expr.alias).zip(
                filtered_columns
                    .into_iter()
                    .map(|col| col.slice_range_from(1..)),
            ),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator")
    }
}

#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
pub(super) fn prove_filter<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    c: &[Column<S>],
    s: &'a [bool],
    d: &[Column<S>],
    n: usize,
    m: usize,
) {
    let input_ones = alloc.alloc_slice_fill_copy(n, true);
    let chi = alloc.alloc_slice_fill_copy(n, false);
    chi[..m].fill(true);

    let c_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(c_fold, One::one(), beta, c);
    let d_bar_fold = alloc.alloc_slice_fill_copy(n + 2, alpha);
    fold_columns(d_bar_fold, One::one(), beta, d);

    let c_star = alloc.alloc_slice_copy(c_fold);
    let d_star = alloc.alloc_slice_copy(d_bar_fold);
    d_star[m..].fill(Zero::zero());
    slice_ops::batch_inversion(c_star);
    slice_ops::batch_inversion(&mut d_star[..m]);
    d_bar_fold[n + 1] = S::ZERO;
    d_star[n + 1] = d_star[0];

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(&d_star[1..]);

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_]), Box::new(s)]),
            (-S::one(), vec![Box::new(&d_star[1..])]),
        ],
    );

    // c_fold * c_star - input_ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(input_ones as &[_])]),
        ],
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(&d_star[1..]), Box::new(&d_bar_fold[1..])],
            ),
            (-S::one(), vec![Box::new(&chi[1..])]),
        ],
    );
}

#[test]
fn we_incorrectly_verify_a_basic_filter_with_a_dishonest_prover() {
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let data = owned_table([
        bigint("a", [101, 104, 105, 102, 105]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(t, data, 0);
    let expr = DishonestFilterExec::new(
        cols_expr_plan(t, &["b"], &accessor),
        tab(t),
        equal(column(t, "a", &accessor), const_int128(105_i128)),
    );
    let res =
        VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(&expr, &accessor, &&prover_setup);
    dbg!(
        &res.verify(&expr, &accessor, &&verifier_setup)
            .unwrap()
            .table
    );
}
