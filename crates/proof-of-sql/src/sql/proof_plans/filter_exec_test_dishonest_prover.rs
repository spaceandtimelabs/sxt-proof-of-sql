use super::{filter_exec::prove_filter, OstensibleFilterExec};
use crate::base::database::owned_table_utility::*;
use crate::{
    base::{
        database::{filter_util::*, Column, DataAccessor, OwnedTableTestAccessor, TestAccessor},
        proof::ProofError,
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::{
            ProofBuilder, ProverEvaluate, ProverHonestyMarker, QueryError, ResultBuilder,
            VerifiableQueryResult,
        },
        // Making this explicit to ensure that we don't accidentally use the
        // sparse filter for these tests
        proof_exprs::{
            test_utility::{cols_expr_plan, column, const_int128, equal, tab},
            ProofExpr,
        },
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::One;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExec<C> = OstensibleFilterExec<C, Dishonest>;

impl ProverEvaluate<Curve25519Scalar> for DishonestFilterExec<RistrettoPoint> {
    #[tracing::instrument(
        name = "DishonestFilterExec::result_evaluate",
        level = "debug",
        skip_all
    )]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) -> Vec<Column<'a, Curve25519Scalar>> {
        // 1. selection
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .result_evaluate(builder.table_length(), alloc, accessor)
            })
            .collect();
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        builder.set_table_length(result_len);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        builder.request_post_result_challenges(2);
        filtered_columns
    }

    #[tracing::instrument(
        name = "DishonestFilterExec::prover_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, Curve25519Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) -> Vec<Column<'a, Curve25519Scalar>> {
        // 1. selection
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor))
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
        filtered_columns
    }
}

/// Tamper with the first element of the first column that is a Scalar. This could be changed for different types of tests.
fn tamper_column<'a>(
    alloc: &'a Bump,
    mut columns: Vec<Column<'a, Curve25519Scalar>>,
) -> Vec<Column<'a, Curve25519Scalar>> {
    for column in &mut columns {
        if let Column::Scalar(tampered_column) = column {
            if !tampered_column.is_empty() {
                let tampered_column = alloc.alloc_slice_copy(tampered_column);
                // The following could be changed for different types of tests, but for the simplest one, we will simply increase the first element by 1.
                tampered_column[0] += Curve25519Scalar::one();
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
