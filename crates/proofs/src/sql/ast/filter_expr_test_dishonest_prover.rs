use super::OstensibleFilterExpr;
use crate::{
    base::{
        database::{DataAccessor, TestAccessor},
        proof::ProofError,
    },
    record_batch,
    sql::{
        ast::test_utility::*,
        proof::{
            ProofBuilder, ProverEvaluate, ProverHonestyMarker, QueryError, VerifiableQueryResult,
        },
    },
};
use bumpalo::Bump;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExpr = OstensibleFilterExpr<Dishonest>;

impl ProverEvaluate for DishonestFilterExpr {
    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.prover_evaluate",
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

        // set result indexes
        let mut cnt: usize = 0;
        for b in selection {
            cnt += *b as usize;
        }
        let indexes = alloc.alloc_slice_fill_default::<u64>(cnt);
        cnt = 0;
        for (i, b) in selection.iter().enumerate() {
            if *b {
                indexes[cnt] = i as u64;
                cnt += 1;
            }
        }
        indexes[0] += 1;
        builder.set_result_indexes(indexes);

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
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = equal(t, "a", 5, &accessor);
    let expr = DishonestFilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    assert!(matches!(
        res.verify(&expr, &accessor),
        Err(QueryError::ProofError(ProofError::VerificationError(_)))
    ));
}
