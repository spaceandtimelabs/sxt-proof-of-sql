use super::symbols::Name;

use serde::{Deserialize, Serialize};

/// Representation of a select statement, that is, the only type of queries allowed.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SelectStatement {
    /// the query expression
    pub expr: Box<SetExpression>,
}

/// Representations of base queries
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum TableExpression {
    /// The row set of a given table; possibly providing an alias
    TableRef {
        /// the qualified table name
        table: Name,
        /// the qualified table namespace
        namespace: Option<Name>,
    },
}

/// Representation of result columns in a select statement
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ResultColumns {
    /// All columns ('*')
    All,

    /// Result column specification
    List(Vec<Box<ResultColumn>>),
}

/// Representation of a single result column specification
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ResultColumn {
    /// All columns from a given named schema object
    AllFrom(Name),

    /// An expression
    Expr {
        /// the expression to evaluate
        expr: Box<Expression>,

        /// an optional column name in the resulting row set
        rename: Option<Name>,
    },
}

/// Possible unary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Numeric negation
    Negate,

    /// Logical inversion
    Not,
}

/// Binary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Numeric multiplication
    Multiply,

    /// Numeric division
    Divide,

    /// Numeric addition
    Add,

    /// Numeric subtraction
    Subtract,

    /// Logical and
    And,

    /// Logical or
    Or,
}

/// Comparison operators
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ComparisonOperator {
    /// Equality
    Equal,

    /// Inquality
    NotEqual,
}

/// Scalar expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Expression {
    /// a literal value
    Literal(Literal),

    /// a qualified name referring to an attribute of a bound relation
    QualifiedIdentifier(Vec<Name>),

    /// unary operation
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },

    /// Binary operation
    Binary {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Comparison operation
    Comparison {
        op: ComparisonOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

/// Representation of a SetExpression, a collection of rows, each having one or more columns.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum SetExpression {
    /// Query result as `SetExpression`
    Query {
        columns: ResultColumns,
        from: Vec<Box<TableExpression>>,
        where_expr: Option<Box<Expression>>,
    },
}

/// Literal values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Literal {
    /// String literal
    StringLiteral(String),

    /// Numeric literal
    NumericLiteral(i64),
}

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}
