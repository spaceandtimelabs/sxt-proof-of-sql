use super::{BoolExpr, FilterResultExpr, TableExpr};

use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};
use crate::base::math::log2_up;
use crate::sql::proof::{ProofBuilder, ProofCounts, QueryExpr, VerificationBuilder};

use bumpalo::Bump;
use std::cmp;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct FilterExpr {
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: Box<dyn BoolExpr>,
}

impl FilterExpr {
    /// Creates a new filter expression
    pub fn new(
        results: Vec<FilterResultExpr>,
        table: TableExpr,
        where_clause: Box<dyn BoolExpr>,
    ) -> Self {
        Self {
            results,
            table,
            where_clause,
        }
    }
}

impl QueryExpr for FilterExpr {
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor) {
        let n = accessor.get_length(&self.table.name);
        counts.table_length = n;
        if n > 0 {
            counts.sumcheck_variables = cmp::max(log2_up(n), 1);
        } else {
            counts.sumcheck_variables = 0;
        }
        self.where_clause.count(counts);
        for expr in self.results.iter() {
            expr.count(counts);
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) {
        // evaluate where clause
        let selection =
            self.where_clause
                .prover_evaluate(builder, alloc, &self.table, counts, accessor);

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
        builder.set_result_indexes(indexes);

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, &self.table, counts, accessor, selection);
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) {
        let selection_eval =
            self.where_clause
                .verifier_evaluate(builder, &self.table, counts, accessor);
        for expr in self.results.iter() {
            expr.verifier_evaluate(builder, &self.table, counts, accessor, &selection_eval);
        }
    }
}
