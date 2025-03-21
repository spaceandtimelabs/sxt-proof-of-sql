use super::{
    DynProofPlan, EmptyExec, FilterExec, GroupByExec, ProjectionExec, SliceExec, SortMergeJoinExec,
    TableExec, UnionExec,
};
use crate::{
    base::database::{ColumnField, ColumnType, TableRef},
    sql::proof_exprs::{
        AliasedDynProofExpr, ColumnExpr, DynProofExpr, IsTrueExpr, ProofExpr, TableExpr,
    },
};
use sqlparser::ast::Ident;

pub fn column_field(name: &str, column_type: ColumnType) -> ColumnField {
    ColumnField::new(name.into(), column_type)
}

pub fn empty_exec() -> DynProofPlan {
    DynProofPlan::Empty(EmptyExec::new())
}

pub fn table_exec(table_ref: TableRef, schema: Vec<ColumnField>) -> DynProofPlan {
    DynProofPlan::Table(TableExec::new(table_ref, schema))
}

pub fn projection(results: Vec<AliasedDynProofExpr>, input: DynProofPlan) -> DynProofPlan {
    DynProofPlan::Projection(ProjectionExec::new(results, Box::new(input)))
}

pub fn filter(
    results: Vec<AliasedDynProofExpr>,
    table: TableExpr,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    // Ensure the WHERE clause is wrapped in IsTrueExpr for proper NULL handling
    let wrapped_where_clause = if where_clause.data_type() == ColumnType::Boolean {
        // Only wrap if it's a boolean expression and not already an IS TRUE expression
        match &where_clause {
            DynProofExpr::IsTrue(_) => where_clause, // Already wrapped
            _ => DynProofExpr::IsTrue(IsTrueExpr::new(Box::new(where_clause))),
        }
    } else {
        // Non-boolean expressions should have been caught earlier
        where_clause
    };

    DynProofPlan::Filter(FilterExec::new(results, table, wrapped_where_clause))
}

/// # Panics
///
/// Will panic if `count_alias` cannot be parsed as a valid identifier.
pub fn group_by(
    group_by_exprs: Vec<ColumnExpr>,
    sum_expr: Vec<AliasedDynProofExpr>,
    count_alias: &str,
    table: TableExpr,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    DynProofPlan::GroupBy(GroupByExec::new(
        group_by_exprs,
        sum_expr,
        count_alias.into(),
        table,
        where_clause,
    ))
}

pub fn slice_exec(input: DynProofPlan, skip: usize, fetch: Option<usize>) -> DynProofPlan {
    DynProofPlan::Slice(SliceExec::new(Box::new(input), skip, fetch))
}

pub fn union_exec(inputs: Vec<DynProofPlan>, schema: Vec<ColumnField>) -> DynProofPlan {
    DynProofPlan::Union(UnionExec::new(inputs, schema))
}

pub fn sort_merge_join(
    left: DynProofPlan,
    right: DynProofPlan,
    left_join_column_indexes: Vec<usize>,
    right_join_column_indexes: Vec<usize>,
    result_idents: Vec<Ident>,
) -> DynProofPlan {
    DynProofPlan::SortMergeJoin(SortMergeJoinExec::new(
        Box::new(left),
        Box::new(right),
        left_join_column_indexes,
        right_join_column_indexes,
        result_idents,
    ))
}
