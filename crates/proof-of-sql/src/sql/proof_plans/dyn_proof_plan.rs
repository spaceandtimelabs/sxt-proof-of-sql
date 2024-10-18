use super::{FilterExec, GroupByExec, ProjectionExec};
use crate::{
    base::{commitment::Commitment, database::Column, map::IndexSet},
    sql::proof::{ProofPlan, ProverEvaluate},
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DynProofPlan<C: Commitment> {
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
    /// ```
    Projection(ProjectionExec<C>),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
    ///         SUM(<sum_expr1>.0) as <sum_expr1>.1, ..., SUM(<sum_exprN>.0) as <sum_exprN>.1,
    ///         COUNT(*) as count_alias
    ///     FROM <table>
    ///     WHERE <where_clause>
    ///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
    /// ```
    GroupBy(GroupByExec<C>),
    /// Provable expressions for queries of the form, where the result is sent in a dense form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    Filter(FilterExec<C>),
}

impl<C: Commitment> ProofPlan<C> for DynProofPlan<C> {
    fn count(
        &self,
        builder: &mut crate::sql::proof::CountBuilder,
        accessor: &dyn crate::base::database::MetadataAccessor,
    ) -> Result<(), crate::base::proof::ProofError> {
        match self {
            DynProofPlan::Projection(expr) => expr.count(builder, accessor),
            DynProofPlan::GroupBy(expr) => expr.count(builder, accessor),
            DynProofPlan::Filter(expr) => expr.count(builder, accessor),
        }
    }

    fn get_offset(&self, accessor: &dyn crate::base::database::MetadataAccessor) -> usize {
        match self {
            DynProofPlan::Projection(expr) => expr.get_offset(accessor),
            DynProofPlan::GroupBy(expr) => expr.get_offset(accessor),
            DynProofPlan::Filter(expr) => expr.get_offset(accessor),
        }
    }

    #[tracing::instrument(name = "DynProofPlan::verifier_evaluate", level = "debug", skip_all)]
    fn verifier_evaluate(
        &self,
        builder: &mut crate::sql::proof::VerificationBuilder<C>,
        accessor: &dyn crate::base::database::CommitmentAccessor<C>,
        result: Option<&crate::base::database::OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, crate::base::proof::ProofError> {
        match self {
            DynProofPlan::Projection(expr) => expr.verifier_evaluate(builder, accessor, result),
            DynProofPlan::GroupBy(expr) => expr.verifier_evaluate(builder, accessor, result),
            DynProofPlan::Filter(expr) => expr.verifier_evaluate(builder, accessor, result),
        }
    }

    fn get_column_result_fields(&self) -> Vec<crate::base::database::ColumnField> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_column_result_fields(),
            DynProofPlan::GroupBy(expr) => expr.get_column_result_fields(),
            DynProofPlan::Filter(expr) => expr.get_column_result_fields(),
        }
    }

    fn get_column_references(&self) -> IndexSet<crate::base::database::ColumnRef> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_column_references(),
            DynProofPlan::GroupBy(expr) => expr.get_column_references(),
            DynProofPlan::Filter(expr) => expr.get_column_references(),
        }
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for DynProofPlan<C> {
    fn get_input_lengths<'a>(
        &self,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) -> Vec<usize> {
        match self {
            DynProofPlan::Projection(expr) => expr.get_input_lengths(alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.get_input_lengths(alloc, accessor),
            DynProofPlan::Filter(expr) => expr.get_input_lengths(alloc, accessor),
        }
    }

    fn get_output_length<'a>(
        &self,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) -> usize {
        match self {
            DynProofPlan::Projection(expr) => expr.get_output_length(alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.get_output_length(alloc, accessor),
            DynProofPlan::Filter(expr) => expr.get_output_length(alloc, accessor),
        }
    }

    #[tracing::instrument(name = "DynProofPlan::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        input_length: usize,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        match self {
            DynProofPlan::Projection(expr) => expr.result_evaluate(input_length, alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.result_evaluate(input_length, alloc, accessor),
            DynProofPlan::Filter(expr) => expr.result_evaluate(input_length, alloc, accessor),
        }
    }

    fn first_round_evaluate(&self, builder: &mut crate::sql::proof::FirstRoundBuilder) {
        match self {
            DynProofPlan::Projection(expr) => expr.first_round_evaluate(builder),
            DynProofPlan::GroupBy(expr) => expr.first_round_evaluate(builder),
            DynProofPlan::Filter(expr) => expr.first_round_evaluate(builder),
        }
    }

    #[tracing::instrument(name = "DynProofPlan::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a>(
        &self,
        builder: &mut crate::sql::proof::FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        match self {
            DynProofPlan::Projection(expr) => expr.final_round_evaluate(builder, alloc, accessor),
            DynProofPlan::GroupBy(expr) => expr.final_round_evaluate(builder, alloc, accessor),
            DynProofPlan::Filter(expr) => expr.final_round_evaluate(builder, alloc, accessor),
        }
    }
}
