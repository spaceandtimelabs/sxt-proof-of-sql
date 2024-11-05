use super::{FilterExec, GroupByExec, ProjectionExec};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable, TableRef,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        VerificationBuilder,
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DynProofPlan {
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
    /// ```
    Projection(ProjectionExec),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
    ///         SUM(<sum_expr1>.0) as <sum_expr1>.1, ..., SUM(<sum_exprN>.0) as <sum_exprN>.1,
    ///         COUNT(*) as count_alias
    ///     FROM <table>
    ///     WHERE <where_clause>
    ///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
    /// ```
    GroupBy(GroupByExec),
    /// Provable expressions for queries of the form, where the result is sent in a dense form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    Filter(FilterExec),
}

impl ProofPlan for DynProofPlan {
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        match self {
            DynProofPlan::Projection(expr) => expr.count(builder, accessor),
            DynProofPlan::GroupBy(expr) => expr.count(builder, accessor),
            DynProofPlan::Filter(expr) => expr.count(builder, accessor),
        }
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        match self {
            DynProofPlan::Projection(expr) => expr.get_length(accessor),
            DynProofPlan::GroupBy(expr) => expr.get_length(accessor),
            DynProofPlan::Filter(expr) => expr.get_length(accessor),
        }
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        match self {
            DynProofPlan::Projection(expr) => expr.get_offset(accessor),
            DynProofPlan::GroupBy(expr) => expr.get_offset(accessor),
            DynProofPlan::Filter(expr) => expr.get_offset(accessor),
        }
    }

    #[tracing::instrument(name = "DynProofPlan::verifier_evaluate", level = "debug", skip_all)]
    fn verifier_evaluate<C: Commitment>(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        match self {
            DynProofPlan::Projection(expr) => expr.verifier_evaluate(builder, accessor, result),
            DynProofPlan::GroupBy(expr) => expr.verifier_evaluate(builder, accessor, result),
            DynProofPlan::Filter(expr) => expr.verifier_evaluate(builder, accessor, result),
        }
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_column_result_fields(),
            DynProofPlan::GroupBy(expr) => expr.get_column_result_fields(),
            DynProofPlan::Filter(expr) => expr.get_column_result_fields(),
        }
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_column_references(),
            DynProofPlan::GroupBy(expr) => expr.get_column_references(),
            DynProofPlan::Filter(expr) => expr.get_column_references(),
        }
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_table_references(),
            DynProofPlan::GroupBy(expr) => expr.get_table_references(),
            DynProofPlan::Filter(expr) => expr.get_table_references(),
        }
    }
}

impl ProverEvaluate for DynProofPlan {
    #[tracing::instrument(name = "DynProofPlan::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        input_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        match self {
            DynProofPlan::Projection(expr) => expr.result_evaluate(input_length, alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.result_evaluate(input_length, alloc, accessor),
            DynProofPlan::Filter(expr) => expr.result_evaluate(input_length, alloc, accessor),
        }
    }

    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        match self {
            DynProofPlan::Projection(expr) => expr.first_round_evaluate(builder),
            DynProofPlan::GroupBy(expr) => expr.first_round_evaluate(builder),
            DynProofPlan::Filter(expr) => expr.first_round_evaluate(builder),
        }
    }

    #[tracing::instrument(name = "DynProofPlan::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        match self {
            DynProofPlan::Projection(expr) => expr.final_round_evaluate(builder, alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.final_round_evaluate(builder, alloc, accessor),
            DynProofPlan::Filter(expr) => expr.final_round_evaluate(builder, alloc, accessor),
        }
    }
}
