//! Modified from Datafusion code. 
use crate::base::database::{ColumnRef, LiteralValue, TableRef};
use alloc::string::String;
use sqlparser::ast::BinaryOperator;

/// Represents a general Proof of SQL expression.
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub enum Expr {
    /// An expression with a specific name.
    Alias(Alias),
    /// A named reference to a qualified field in a schema.
    Column(ColumnRef),
    /// A constant value.
    Literal(LiteralValue),
    /// A binary expression such as "age > 21"
    BinaryExpr(BinaryExpr),
    /// Negation of an expression. The expression's type must be a boolean to make sense.
    Not(Box<Expr>),
    /// arithmetic negation of an expression, the operand must be of a signed numeric data type
    Negative(Box<Expr>),
    /// Represents a reference to all available fields in a specific schema,
    /// with an optional (schema) qualifier.
    ///
    /// This expr has to be resolved to a list of columns before translating logical
    /// plan into physical plan.
    Wildcard { qualifier: Option<TableRef> },
}

/// Alias expression
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub struct Alias {
    pub expr: Box<Expr>,
    pub relation: Option<TableRef>,
    pub name: String,
}

impl Alias {
    /// Create an alias with an optional schema/field qualifier.
    pub fn new(expr: Expr, relation: Option<impl Into<TableRef>>, name: impl Into<String>) -> Self {
        Self {
            expr: Box::new(expr),
            relation: relation.map(|r| r.into()),
            name: name.into(),
        }
    }
}

/// Binary expression
#[derive(Clone, PartialEq, Eq, PartialOrd, Hash, Debug)]
pub struct BinaryExpr {
    /// Left-hand side of the expression
    pub left: Box<Expr>,
    /// The binary operator
    pub op: BinaryOperator,
    /// Right-hand side of the expression
    pub right: Box<Expr>,
}

impl BinaryExpr {
    /// Create a new binary expression
    pub fn new(left: Box<Expr>, op: BinaryOperator, right: Box<Expr>) -> Self {
        Self { left, op, right }
    }
}
