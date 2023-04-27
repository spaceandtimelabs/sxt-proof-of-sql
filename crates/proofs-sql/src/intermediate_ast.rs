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
        columns: Vec<ResultColumnExpr>,
        from: Vec<Box<TableExpression>>,
        where_expr: Option<Box<Expression>>,
        group_by: Vec<Identifier>,
    },
}

/// Representation of a single result column
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResultColumn {
    /// The name of the column
    pub name: Identifier,
    /// The alias of the column
    pub alias: Identifier,
}

/// Representation of a single result column specification
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ResultColumnExpr {
    /// All column expressions
    AllColumns,
    /// A simple column expression
    SimpleColumn(ResultColumn),
    /// An aggregation expression
    AggColumn(AggExpr),
}

/// Representation of an aggregation expression
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum AggExpr {
    /// An aggregation expression associated with max(expr)
    Max(ResultColumn),
    /// An aggregation expression associated with min(expr)
    Min(ResultColumn),
    /// An aggregation expression associated with sum(expr)
    Sum(ResultColumn),
    /// An aggregation expression associated with count(expr)
    Count(ResultColumn),
    /// An aggregation expression associated with count(*)
    CountAll(Identifier),
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
        right: Box<Literal>,
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
    BigInt(i64),
    /// String Literal
    VarChar(String),
}

macro_rules! impl_int_to_literal {
    ($tt:ty) => {
        impl From<$tt> for Literal {
            fn from(val: $tt) -> Self {
                Literal::BigInt(val as i64)
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
