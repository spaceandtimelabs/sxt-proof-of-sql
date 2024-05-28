use super::{
    dense_filter_expr::prove_filter, filter_columns, OstensibleDenseFilterExpr, ProvableExpr,
};
use crate::{
    base::{
        database::{Column, DataAccessor, OwnedTableTestAccessor, TestAccessor},
        proof::ProofError,
        scalar::Curve25519Scalar,
    },
    owned_table,
    sql::{
        // Making this explicit to ensure that we don't accidentally use the
        // sparse filter for these tests
        ast::test_utility::{cols_expr, column, const_int128, equal, tab},
        proof::{
            Indexes, ProofBuilder, ProverEvaluate, ProverHonestyMarker, QueryError, ResultBuilder,
            VerifiableQueryResult,
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
type DishonestDenseFilterExpr<C> = OstensibleDenseFilterExpr<C, Dishonest>;

impl ProverEvaluate<Curve25519Scalar> for DishonestDenseFilterExpr<RistrettoPoint> {
    #[tracing::instrument(
        name = "DishonestDenseFilterExpr::result_evaluate",
        level = "debug",
        skip_all
    )]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        // 1. selection
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.result_evaluate(builder.table_length(), alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(result_len as u64)));
        // 4. set filtered_columns
        for col in filtered_columns {
            builder.produce_result_column(col);
        }
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(
        name = "DishonestDenseFilterExpr::prover_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, Curve25519Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        // 1. selection
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        // 2. columns
        let columns = Vec::from_iter(
            self.results
                .iter()
                .map(|expr| expr.prover_evaluate(builder, alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);

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
    }
}

/// Tamper with the first element of the first column that is a Scalar. This could be changed for different types of tests.
fn tamper_column<'a>(
    alloc: &'a Bump,
    mut columns: Vec<Column<'a, Curve25519Scalar>>,
) -> Vec<Column<'a, Curve25519Scalar>> {
    for column in columns.iter_mut() {
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
fn we_fail_to_verify_a_basic_dense_filter_with_a_dishonest_prover() {
    let data = owned_table!(
        "a" => [101_i64, 104, 105, 102, 105],
        "b" => [1_i64, 2, 3, 4, 5],
        "c" => [1_i128, 2, 3, 4, 5],
        "d" => ["1", "2", "3", "4", "5"],
        "e" => [Curve25519Scalar::from(1), 2.into(), 3.into(), 4.into(), 5.into()],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = DishonestDenseFilterExpr::new(
        cols_expr(t, &["b", "c", "d", "e"], &accessor),
        tab(t),
        equal(column(t, "a", &accessor), const_int128(105_i128)),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(matches!(
        res.verify(&expr, &accessor, &()),
        Err(QueryError::ProofError(ProofError::VerificationError(_)))
    ));
}
