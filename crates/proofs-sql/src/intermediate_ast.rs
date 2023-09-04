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
        result_columns: Vec<SelectResultExpr>,
        from: Vec<Box<TableExpression>>,
        where_expr: Option<Box<Expression>>,
        group_by: Vec<Identifier>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SelectResultExpr {
    ALL,
    AliasedResultExpr(AliasedResultExpr),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AliasedResultExpr {
    pub expr: ResultExpr,
    pub alias: Identifier,
}

impl AliasedResultExpr {
    pub fn from_non_agg_expr(expr: Expression, alias: Identifier) -> Self {
        Self {
            expr: ResultExpr::NonAgg(Box::new(expr)),
            alias,
        }
    }

    pub fn try_as_identifier(&self) -> Option<&Identifier> {
        match &self.expr {
            ResultExpr::NonAgg(expr) => match expr.as_ref() {
                Expression::Column(column) => Some(column),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ResultExpr {
    Agg(AggExpr),
    NonAgg(Box<Expression>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum AggExpr {
    /// An aggregation expression associated with max(expr)
    Max(Box<Expression>),
    /// An aggregation expression associated with min(expr)
    Min(Box<Expression>),
    /// An aggregation expression associated with sum(expr)
    Sum(Box<Expression>),
    /// An aggregation expression associated with count(expr)
    Count(Box<Expression>),
    /// An aggregation expression associated with count(*)
    CountALL,
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

/// Binary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    /// Numeric addition
    Add,

    /// Numeric subtraction
    Subtract,

    /// Numeric multiplication
    Multiply,

    /// Logical And
    And,

    /// Logical Or
    Or,

    /// Comparison =
    Equal,
}

/// Possible unary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    /// Logical inversion
    Not,
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
        op: UnaryOperator,
        expr: Box<Expression>,
    },

    /// Binary operation
    Binary {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

/// OrderBy
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OrderBy {
    pub expr: Identifier,
    pub direction: OrderByDirection,
}

/// OrderByDirection values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum OrderByDirection {
    Asc,
    Desc,
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
    /// Numeric Literal
    Int128(i128),
    /// String Literal
    VarChar(String),
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

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}
