use crate::base::database::{ColumnRef, LiteralValue};
use alloc::boxed::Box;
use serde::{Deserialize, Serialize};

/// Enum of column expressions that are either provable or supported in postprocessing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Column
    Column(ColumnRef),
    /// A constant expression
    Literal(LiteralValue),
    /// Binary operation
    Binary {
        /// Left hand side of the binary operation
        left: Box<Expr>,
        /// Right hand side of the binary operation
        right: Box<Expr>,
        /// Binary operator
        op: BinaryOperator,
    },
    /// NOT expression
    Not(Box<Expr>),
}

/// Enum of binary operators we support
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// Equals
    Eq,
    /// Not equals
    NotEq,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Greater than or equals
    GtEq,
    /// Less than or equals
    LtEq,
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Plus
    Plus,
    /// Minus
    Minus,
    /// Multiply
    Multiply,
    /// Divide
    Divide,
}
