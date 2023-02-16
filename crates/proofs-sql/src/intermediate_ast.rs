/***
* These AST nodes are closely following vervolg:
* https://docs.rs/vervolg/latest/vervolg/ast/enum.Statement.html
***/

use serde::{Deserialize, Serialize};

use crate::Identifier;

/// Representation of a SetExpression, a collection of rows, each having one or more columns.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SetExpression {
    /// Query result as `SetExpression`
    Query {
        columns: Vec<Box<ResultColumn>>,
        from: Vec<Box<TableExpression>>,
        where_expr: Option<Box<Expression>>,
    },
}

/// Representation of a single result column specification
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ResultColumn {
    /// All column expressions
    All,
    /// A column expression
    Expr {
        expr: Identifier,
        output_name: Option<Identifier>,
    },
}

/// Representations of base queries
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TableExpression {
    /// The row set of a given table; possibly providing an alias
    Named {
        /// the qualified table Identifier
        table: Identifier,
        schema: Option<Identifier>,
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
        left: Identifier,
        right: i64,
    },

    /// left != right
    /// left <> right
    NotEqual {
        left: Identifier,
        right: i64,
    },
}

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}
