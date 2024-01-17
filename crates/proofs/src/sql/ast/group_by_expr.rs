use super::{aggregate_columns, group_by_util::AggregatedColumns, BoolExpr, ColumnExpr, TableExpr};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
        },
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::proof::{
        CountBuilder, Indexes, ProofBuilder, ProofExpr, ProverEvaluate, ResultBuilder,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::iter::repeat_with;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::DynPartialEq;
use proofs_sql::Identifier;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
///         SUM(<sum_expr1>.0) as <sum_expr1>.1, ..., SUM(<sum_exprN>.0) as <sum_exprN>.1,
///         COUNT(*) as count_alias
///     FROM <table>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize, DynPartialEq)]
pub struct GroupByExpr {
    pub(super) group_by_exprs: Vec<ColumnExpr>,
    pub(super) sum_expr: Vec<(ColumnExpr, ColumnField)>,
    pub(super) count_alias: Identifier,
    pub(super) table: TableExpr,
    pub(super) where_clause: Box<dyn BoolExpr>,
}

impl GroupByExpr {
    /// Creates a new group_by expression.
    pub fn new(
        group_by_exprs: Vec<ColumnExpr>,
        sum_expr: Vec<(ColumnExpr, ColumnField)>,
        count_alias: Identifier,
        table: TableExpr,
        where_clause: Box<dyn BoolExpr>,
    ) -> Self {
        Self {
            group_by_exprs,
            sum_expr,
            table,
            count_alias,
            where_clause,
        }
    }
}

impl ProofExpr for GroupByExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.group_by_exprs.iter() {
            expr.count(builder);
            builder.count_result_columns(1);
        }
        for expr in self.sum_expr.iter() {
            expr.0.count(builder);
            builder.count_result_columns(1);
        }
        builder.count_result_columns(1);
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table.table_ref)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.group_by_expr.verifier_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<(), ProofError> {
        // 1. selection
        let where_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let group_by_evals = Vec::from_iter(
            self.group_by_exprs
                .iter()
                .map(|expr| expr.verifier_evaluate(builder, accessor)),
        );
        let aggregate_evals = Vec::from_iter(
            self.sum_expr
                .iter()
                .map(|expr| expr.0.verifier_evaluate(builder, accessor)),
        );
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns

        let group_by_result_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_result_mle()).take(self.group_by_exprs.len()),
        );
        let sum_result_columns_evals =
            Vec::from_iter(repeat_with(|| builder.consume_result_mle()).take(self.sum_expr.len()));
        let count_column_eval = builder.consume_result_mle();

        // TODO: verify the proof using the above evaluations.

        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut fields = Vec::new();
        for col in self.group_by_exprs.iter() {
            fields.push(col.get_column_field());
        }
        for col in self.sum_expr.iter() {
            fields.push(col.1);
        }
        fields.push(ColumnField::new(self.count_alias, ColumnType::BigInt));
        fields
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        let mut columns = HashSet::new();

        for col in self.group_by_exprs.iter() {
            columns.insert(col.get_column_reference());
        }
        for col in self.sum_expr.iter() {
            columns.insert(col.0.get_column_reference());
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

impl ProverEvaluate for GroupByExpr {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) {
        // 1. selection
        let selection = self
            .where_clause
            .result_evaluate(builder.table_length(), alloc, accessor);
        // 2. columns
        let group_by_columns = Vec::from_iter(
            self.group_by_exprs
                .iter()
                .map(|expr| expr.result_evaluate(accessor)),
        );
        let sum_columns = Vec::from_iter(
            self.sum_expr
                .iter()
                .map(|expr| expr.0.result_evaluate(accessor)),
        );
        // Compute filtered_columns and indexes
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, selection)
            .expect("columns should be aggregatable");
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(count_column.len() as u64)));
        // 4. set filtered_columns
        for col in group_by_result_columns {
            builder.produce_result_column(col);
        }
        for col in sum_result_columns {
            builder.produce_result_column(col);
        }
        builder.produce_result_column(count_column);
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.group_by_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) {
        // 1. selection
        let selection = self.where_clause.prover_evaluate(builder, alloc, accessor);
        // 2. columns
        let group_by_columns = Vec::from_iter(
            self.group_by_exprs
                .iter()
                .map(|expr| expr.prover_evaluate(builder, accessor)),
        );
        let sum_columns = Vec::from_iter(
            self.sum_expr
                .iter()
                .map(|expr| expr.0.prover_evaluate(builder, accessor)),
        );
        // Compute filtered_columns and indexes
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, selection)
            .expect("columns should be aggregatable");

        // TODO: produce the proof using the above columns.
    }
}
