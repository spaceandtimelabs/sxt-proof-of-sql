/***
* These AST nodes are closely following vervolg:
* https://docs.rs/vervolg/latest/vervolg/ast/enum.Statement.html
***/

use super::symbols::Name;
use serde::{Deserialize, Serialize};

/// Representation of a select statement, that is, the only type of queries allowed.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SelectStatement {
    /// the query expression
    pub expr: Box<SetExpression>,
}

/// Representation of a SetExpression, a collection of rows, each having one or more columns.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SetExpression {
    /// Query result as `SetExpression`
    Query {
        columns: Vec<Box<ResultColumn>>,
        from: Vec<Box<TableExpression>>,
        where_expr: Box<Expression>,
    },
}

/// Representation of a single result column specification
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ResultColumn {
    /// An expression
    Expr {
        expr: Name,
        output_name: Option<Name>,
    },
}

/// Representations of base queries
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TableExpression {
    /// The row set of a given table; possibly providing an alias
    Named {
        /// the qualified table name
        table: Name,
        namespace: Option<Name>,
    },
}

/// Boolean expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    // not (expr)
    Not {
        expr: Box<Expression>,
    },

    // left and right
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // left or right
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// left == right
    Equal {
        left: Name,
        right: i64,
    },

    /// left != right
    /// left <> right
    NotEqual {
        left: Name,
        right: i64,
    },
}

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}
