use super::{DenseFilterExpr, FilterExpr, GroupByExpr};
use crate::{
    base::commitment::Commitment,
    sql::proof::{ProofExpr, ProverEvaluate},
};
use serde::{Deserialize, Serialize};

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProofPlan<C: Commitment> {
    /// Provable expressions for queries of the form, where the result is sent in a sparse form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    Filter(FilterExpr<C>),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
    ///         SUM(<sum_expr1>.0) as <sum_expr1>.1, ..., SUM(<sum_exprN>.0) as <sum_exprN>.1,
    ///         COUNT(*) as count_alias
    ///     FROM <table>
    ///     WHERE <where_clause>
    ///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
    /// ```
    GroupBy(GroupByExpr<C>),
    /// Provable expressions for queries of the form, where the result is sent in a dense form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    DenseFilter(DenseFilterExpr<C>),
}

impl<C: Commitment> ProofExpr<C> for ProofPlan<C> {
    fn count(
        &self,
        builder: &mut crate::sql::proof::CountBuilder,
        accessor: &dyn crate::base::database::MetadataAccessor,
    ) -> Result<(), crate::base::proof::ProofError> {
        match self {
            ProofPlan::Filter(expr) => expr.count(builder, accessor),
            ProofPlan::GroupBy(expr) => expr.count(builder, accessor),
            ProofPlan::DenseFilter(expr) => expr.count(builder, accessor),
        }
    }

    fn get_length(&self, accessor: &dyn crate::base::database::MetadataAccessor) -> usize {
        match self {
            ProofPlan::Filter(expr) => expr.get_length(accessor),
            ProofPlan::GroupBy(expr) => expr.get_length(accessor),
            ProofPlan::DenseFilter(expr) => expr.get_length(accessor),
        }
    }

    fn get_offset(&self, accessor: &dyn crate::base::database::MetadataAccessor) -> usize {
        match self {
            ProofPlan::Filter(expr) => expr.get_offset(accessor),
            ProofPlan::GroupBy(expr) => expr.get_offset(accessor),
            ProofPlan::DenseFilter(expr) => expr.get_offset(accessor),
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut crate::sql::proof::VerificationBuilder<C>,
        accessor: &dyn crate::base::database::CommitmentAccessor<C>,
    ) -> Result<(), crate::base::proof::ProofError> {
        match self {
            ProofPlan::Filter(expr) => expr.verifier_evaluate(builder, accessor),
            ProofPlan::GroupBy(expr) => expr.verifier_evaluate(builder, accessor),
            ProofPlan::DenseFilter(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_result_fields(&self) -> Vec<crate::base::database::ColumnField> {
        match self {
            ProofPlan::Filter(expr) => expr.get_column_result_fields(),
            ProofPlan::GroupBy(expr) => expr.get_column_result_fields(),
            ProofPlan::DenseFilter(expr) => expr.get_column_result_fields(),
        }
    }

    fn get_column_references(&self) -> std::collections::HashSet<crate::base::database::ColumnRef> {
        match self {
            ProofPlan::Filter(expr) => expr.get_column_references(),
            ProofPlan::GroupBy(expr) => expr.get_column_references(),
            ProofPlan::DenseFilter(expr) => expr.get_column_references(),
        }
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for ProofPlan<C> {
    fn result_evaluate<'a>(
        &self,
        builder: &mut crate::sql::proof::ResultBuilder<'a>,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) {
        match self {
            ProofPlan::Filter(expr) => expr.result_evaluate(builder, alloc, accessor),
            ProofPlan::GroupBy(expr) => expr.result_evaluate(builder, alloc, accessor),
            ProofPlan::DenseFilter(expr) => expr.result_evaluate(builder, alloc, accessor),
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut crate::sql::proof::ProofBuilder<'a, C::Scalar>,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<C::Scalar>,
    ) {
        match self {
            ProofPlan::Filter(expr) => expr.prover_evaluate(builder, alloc, accessor),
            ProofPlan::GroupBy(expr) => expr.prover_evaluate(builder, alloc, accessor),
            ProofPlan::DenseFilter(expr) => expr.prover_evaluate(builder, alloc, accessor),
        }
    }
}
