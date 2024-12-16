use super::{EmptyExec, FilterExec, GroupByExec, ProjectionExec, SliceExec, TableExec, UnionExec};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[enum_dispatch::enum_dispatch]
pub enum DynProofPlan {
    /// Source [`ProofPlan`] for (sub)queries without table source such as `SELECT "No table here" as msg;`
    Empty(EmptyExec),
    /// Source [`ProofPlan`] for (sub)queries with table source such as `SELECT col from tab;`
    Table(TableExec),
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
    /// `ProofPlan` for queries of the form
    /// ```ignore
    ///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
    /// ```
    Slice(SliceExec),
    /// `ProofPlan` for queries of the form
    /// ```ignore
    ///     <ProofPlan>
    ///     UNION ALL
    ///     <ProofPlan>
    ///     ...
    ///     UNION ALL
    ///     <ProofPlan>
    /// ```
    Union(UnionExec),
}
