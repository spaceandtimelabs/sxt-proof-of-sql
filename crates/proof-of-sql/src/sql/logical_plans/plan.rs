use super::Expr;
use crate::base::database::{ColumnField, TableRef};
use alloc::{boxed::Box, vec::Vec};
use serde::{Deserialize, Serialize};

/// Enum of logical plans that are either provable or supported in postprocessing
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum LogicalPlan {
    /// Empty
    Empty,
    /// Table scan
    TableScan(TableRef),
    /// Projection
    Projection(Projection),
    /// Filter
    Filter(Filter),
    /// Aggregate
    Aggregate(Aggregate),
    /// Sort
    Sort(Sort),
    /// Slice
    Slice(Slice),
    /// Join
    Join(Join),
    /// Union
    Union(Union),
}

/// Projection
/// e.g. SELECT a, b FROM t
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Projection {
    /// Input plan
    pub input: Box<LogicalPlan>,
    /// Projection expressions
    pub expr: Vec<Expr>,
}

/// Filter
/// e.g. WHERE a > 5
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Filter {
    /// Input plan
    pub input: Box<LogicalPlan>,
    /// Filter expression
    pub filter: Expr,
}

/// Aggregate
/// e.g. SELECT a, COUNT(b) FROM t GROUP BY a
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Aggregate {
    /// Input plan
    pub input: Box<LogicalPlan>,
    /// Group by
    pub group_by: Vec<Expr>,
    /// Aggregate expressions
    pub aggr_expr: Vec<Expr>,
    /// Output schema
    pub schema: Vec<ColumnField>,
}

/// Sort
/// e.g. ORDER BY a ASC, b DESC
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Sort {
    /// Input plan
    pub input: Box<LogicalPlan>,
    /// Sort expressions
    pub expr: Vec<SortExpr>,
}

/// Sort expression
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SortExpr {
    /// Expression
    pub expr: Expr,
    /// Direction
    pub asc: bool,
}

/// Slice
/// e.g. LIMIT 5 OFFSET 10
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Slice {
    /// Input plan
    pub input: Box<LogicalPlan>,
    /// Maximum number of rows to return. None = no limit
    pub limit: Option<u64>,
    /// Offset value
    pub offset: i64,
}

/// Join
/// e.g. SELECT t1.a, t1.b, t2.c FROM t1 JOIN t2 ON t1.a = t2.a
#[allow(clippy::struct_field_names)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Join {
    /// Left input plan
    pub left: Box<LogicalPlan>,
    /// Right input plan
    pub right: Box<LogicalPlan>,
    /// Equijoin condition
    pub on: Vec<(Expr, Expr)>,
    /// Join type
    pub join_type: JoinType,
    /// Output schema
    pub schema: Vec<ColumnField>,
}

/// Join type
///
/// Currently only supports inner join
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum JoinType {
    /// Inner join
    Inner,
}

/// Union
/// e.g. SELECT a, b FROM t1 UNION ALL SELECT a, b FROM t2
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Union {
    /// Input plans
    pub inputs: Vec<LogicalPlan>,
    /// Schema of the output
    pub schema: Vec<ColumnField>,
}
