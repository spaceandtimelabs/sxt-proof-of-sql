use crate::sql::proof::{ProofBuilder, ProofCounts, QueryExpr, VerificationBuilder};

use crate::sql::ast::{BoolExpr, FilterResultExpr, TableExpr};

use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};
use bumpalo::Bump;

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
    #[allow(unused_variables)]
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor) {
        todo!();
    }

    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) {
        todo!();
    }

    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) {
        todo!();
    }
}
