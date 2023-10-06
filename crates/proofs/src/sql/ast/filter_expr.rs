use super::{BoolExpr, FilterResultExpr, TableExpr};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor},
        proof::ProofError,
    },
    sql::proof::{CountBuilder, ProofBuilder, ProofExpr, VerificationBuilder},
};
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct FilterExpr {
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: Box<dyn BoolExpr>,
}

impl FilterExpr {
    /// Creates a new filter expression.
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

    /// Returns the result expressions.
    pub fn get_results(&self) -> &[FilterResultExpr] {
        &self.results[..]
    }
}

impl ProofExpr for FilterExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.results.iter() {
            expr.count(builder);
        }
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table.table_ref)
    }

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
        builder.set_result_indexes(indexes);

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, accessor, selection);
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.verifier_evaluate",
        level = "debug",
        skip_all
    )]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<(), ProofError> {
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        for expr in self.results.iter() {
            expr.verifier_evaluate(builder, accessor, &selection_eval);
        }
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut columns = Vec::with_capacity(self.results.len());
        for col in self.results.iter() {
            columns.push(col.get_column_field());
        }
        columns
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        let mut columns = HashSet::new();

        for col in self.results.iter() {
            columns.insert(col.get_column_reference());
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}
