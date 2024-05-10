//! TODO: add docs
/***
* These AST nodes are closely following vervolg:
* https://docs.rs/vervolg/latest/vervolg/ast/enum.Statement.html
***/

use crate::{decimal_unknown::DecimalUnknown, Identifier};
use serde::{Deserialize, Serialize};

/// Representation of a SetExpression, a collection of rows, each having one or more columns.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SetExpression {
    /// Query result as `SetExpression`
    Query {
        /// TODO: add docs
        result_exprs: Vec<SelectResultExpr>,
        /// TODO: add docs
        from: Vec<Box<TableExpression>>,
        /// TODO: add docs
        where_expr: Option<Box<Expression>>,
        /// TODO: add docs
        group_by: Vec<Identifier>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// TODO: add docs
pub enum SelectResultExpr {
    /// TODO: add docs
    ALL,
    /// TODO: add docs
    AliasedResultExpr(AliasedResultExpr),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// TODO: add docs
pub struct AliasedResultExpr {
    /// TODO: add docs
    pub expr: Box<Expression>,
    /// TODO: add docs
    pub alias: Identifier,
}

impl AliasedResultExpr {
    /// TODO: add docs
    pub fn new(expr: Expression, alias: Identifier) -> Self {
        Self {
            expr: Box::new(expr),
            alias,
        }
    }

    /// TODO: add docs
    pub fn try_as_identifier(&self) -> Option<&Identifier> {
        match self.expr.as_ref() {
            Expression::Column(column) => Some(column),
            _ => None,
        }
    }
}

/// Representations of base queries
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TableExpression {
    /// The row set of a given table; possibly providing an alias
    Named {
        /// the qualified table Identifier
        table: Identifier,
        /// TODO: add docs
        schema: Option<Identifier>,
    },
}

/// Binary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    /// Numeric addition
    Add,

    /// Numeric subtraction
    Subtract,

    /// Numeric multiplication
    Multiply,

    /// Numeric division
    Division,

    /// Logical And
    And,

    /// Logical Or
    Or,

    /// Comparison =
    Equal,

    /// Comparison <=
    LessThanOrEqual,

    /// Comparison >=
    GreaterThanOrEqual,
}

/// Possible unary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    /// Logical inversion
    Not,
}

// Aggregation operators
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// TODO: add docs
pub enum AggregationOperator {
    /// TODO: add docs
    Max,
    /// TODO: add docs
    Min,
    /// TODO: add docs
    Sum,
    /// TODO: add docs
    Count,
    /// TODO: add docs
    First,
}

impl std::fmt::Display for AggregationOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregationOperator::Max => write!(f, "max"),
            AggregationOperator::Min => write!(f, "min"),
            AggregationOperator::Sum => write!(f, "sum"),
            AggregationOperator::Count => write!(f, "count"),
            AggregationOperator::First => write!(f, "first"),
        }
    }
}

/// Boolean Expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    /// Literal
    Literal(Literal),

    /// Column
    Column(Identifier),

    /// Unary operation
    Unary {
        /// TODO: add docs
        op: UnaryOperator,
        /// TODO: add docs
        expr: Box<Expression>,
    },

    /// Binary operation
    Binary {
        /// TODO: add docs
        op: BinaryOperator,
        /// TODO: add docs
        left: Box<Expression>,
        /// TODO: add docs
        right: Box<Expression>,
    },

    /// * expression
    Wildcard,

    /// Aggregation operation
    Aggregation {
        /// TODO: add docs
        op: AggregationOperator,
        /// TODO: add docs
        expr: Box<Expression>,
    },
}

impl Expression {
    /// TODO: add docs
    pub fn sum(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Sum,
            expr: Box::new(self),
        })
    }

    /// TODO: add docs
    pub fn max(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Max,
            expr: Box::new(self),
        })
    }

    /// TODO: add docs
    pub fn min(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Min,
            expr: Box::new(self),
        })
    }

    /// TODO: add docs
    pub fn count(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Count,
            expr: Box::new(self),
        })
    }

    /// TODO: add docs
    pub fn first(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::First,
            expr: Box::new(self),
        })
    }
}

/// OrderBy
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OrderBy {
    /// TODO: add docs
    pub expr: Identifier,
    /// TODO: add docs
    pub direction: OrderByDirection,
}

/// OrderByDirection values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderByDirection {
    /// TODO: add docs
    Asc,
    /// TODO: add docs
    Desc,
}

impl std::fmt::Display for OrderByDirection {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrderByDirection::Asc => write!(f, "asc"),
            OrderByDirection::Desc => write!(f, "desc"),
        }
    }
}

/// Limits for a limit clause
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Slice {
    /// number of rows to return
    ///
    /// if u64::MAX, specify all rows
    pub number_rows: u64,

    /// number of rows to skip
    ///
    /// if 0, specify the first row as starting point
    /// if negative, specify the offset from the end
    /// (e.g. -1 is the last row, -2 is the second to last row, etc.)
    pub offset_value: i64,
}

/// Literal values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    /// Boolean Literal
    Boolean(bool),
    /// Numeric Literal
    Int128(i128),
    /// String Literal
    VarChar(String),
    /// Decimal Literal
    Decimal(DecimalUnknown),
}

impl From<bool> for Literal {
    fn from(val: bool) -> Self {
        Literal::Boolean(val)
    }
}

macro_rules! impl_int_to_literal {
    ($tt:ty) => {
        impl From<$tt> for Literal {
            fn from(val: $tt) -> Self {
                Literal::Int128(val as i128)
            }
        }
    };
}

impl_int_to_literal!(i8);
impl_int_to_literal!(u8);
impl_int_to_literal!(i16);
impl_int_to_literal!(u16);
impl_int_to_literal!(i32);
impl_int_to_literal!(u32);
impl_int_to_literal!(i64);
impl_int_to_literal!(i128);

macro_rules! impl_string_to_literal {
    ($tt:ty) => {
        impl From<$tt> for Literal {
            fn from(val: $tt) -> Self {
                Literal::VarChar(val.into())
            }
        }
    };
}

impl_string_to_literal!(&str);
impl_string_to_literal!(String);

impl From<DecimalUnknown> for Literal {
    fn from(val: DecimalUnknown) -> Self {
        Literal::Decimal(val)
    }
}

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}
