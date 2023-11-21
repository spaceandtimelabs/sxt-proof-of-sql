use super::OstensibleDenseFilterExpr;
use crate::{
    base::{
        database::{DataAccessor, RecordBatchTestAccessor, TestAccessor},
        proof::ProofError,
    },
    record_batch,
    sql::{
        // Making this explicit to ensure that we don't accidentally use the
        // sparse filter for these tests
        ast::test_utility::{cols_expr, equal, tab},
        proof::{
            Indexes, ProofBuilder, ProverEvaluate, ProverHonestyMarker, QueryError, ResultBuilder,
            VerifiableQueryResult,
        },
    },
};
use bumpalo::Bump;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestDenseFilterExpr = OstensibleDenseFilterExpr<Dishonest>;

impl ProverEvaluate for DishonestDenseFilterExpr {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        // evaluate where clause
        let selection = self
            .where_clause
            .result_evaluate(builder.table_length(), alloc, accessor);

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
        name = "proofs.sql.ast.dense_filter_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        // evaluate where clause
        let selection = self.where_clause.prover_evaluate(builder, alloc, accessor);

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, accessor, selection);
        }
    }
}

#[test]
fn we_fail_to_verify_a_basic_dense_filter_with_a_dishonest_prover() {
    let data = record_batch!(
        "a" => [1_i64, 4_i64, 5_i64, 2_i64, 5_i64],
        "b" => [1_i64, 2, 3, 4, 5],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = RecordBatchTestAccessor::new_empty();
    accessor.add_table(t, data, 0);
    let where_clause = equal(t, "a", 5, &accessor);
    let expr = DishonestDenseFilterExpr::new(cols_expr(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    assert!(matches!(
        res.verify(&expr, &accessor),
        Err(QueryError::ProofError(ProofError::VerificationError(_)))
    ));
}
