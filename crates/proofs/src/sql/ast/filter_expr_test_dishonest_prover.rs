use super::{OstensibleFilterExpr, ProvableExpr};
use crate::{
    base::{
        database::{Column, DataAccessor, RecordBatchTestAccessor, TestAccessor},
        proof::ProofError,
        scalar::Curve25519Scalar,
    },
    record_batch,
    sql::{
        ast::test_utility::*,
        proof::{
            Indexes, ProofBuilder, ProverEvaluate, ProverHonestyMarker, QueryError, ResultBuilder,
            VerifiableQueryResult,
        },
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExpr = OstensibleFilterExpr<RistrettoPoint, Dishonest>;

impl ProverEvaluate<Curve25519Scalar> for DishonestFilterExpr {
    #[tracing::instrument(
        name = "DishonestFilterExpr::result_evaluate",
        level = "debug",
        skip_all
    )]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        // evaluate where clause
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause
                .result_evaluate(builder.table_length(), alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // set result indexes
        let mut indexes: Vec<_> = selection
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| i as u64)
            .collect();
        indexes[0] += 1;
        builder.set_result_indexes(Indexes::Sparse(indexes));
    }

    #[tracing::instrument(
        name = "DishonestFilterExpr::prover_evaluate",
        level = "debug",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, Curve25519Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        // evaluate where clause
        let selection_column: Column<'a, Curve25519Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, accessor, selection);
        }
    }
}

#[test]
fn we_fail_to_verify_a_basic_filter_with_a_dishonest_prover() {
    let data = record_batch!(
        "a" => [1_i64, 4_i64, 5_i64, 2_i64, 5_i64],
        "b" => [1_i64, 2, 3, 4, 5],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = equal(column(t, "a", &accessor), const_int128(5_i128));
    let expr = DishonestFilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(matches!(
        res.verify(&expr, &accessor, &()),
        Err(QueryError::ProofError(ProofError::VerificationError(_)))
    ));
}
