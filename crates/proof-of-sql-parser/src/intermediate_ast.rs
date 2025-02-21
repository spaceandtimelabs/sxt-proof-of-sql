//! This module contains the AST nodes for the intermediate representation of a Proof of SQL query.
/***
* These AST nodes are closely following vervolg:
* https://docs.rs/vervolg/latest/vervolg/ast/enum.Statement.html
***/

use crate::{posql_time::PoSQLTimestamp, Identifier};
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(test)]
use alloc::vec;
use bigdecimal::BigDecimal;
use core::{
    fmt,
    fmt::{Display, Formatter},
    hash::Hash,
};
use serde::{Deserialize, Serialize};

/// Representation of a `SetExpression`, a collection of rows, each having one or more columns.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum SetExpression {
    /// Query result as `SetExpression`
    Query {
        /// Result expressions e.g. `a` and `b` in `SELECT a, b FROM table`
        result_exprs: Vec<SelectResultExpr>,
        /// Table expression e.g. `table` in `SELECT a, b FROM table`
        from: Vec<Box<TableExpression>>,
        /// Filter expression e.g. `a > 5` in `SELECT a, b FROM table WHERE a > 5`
        /// If None, no filter is applied
        where_expr: Option<Box<Expression>>,
        /// Group by expressions e.g. `a` in `SELECT a, COUNT(*) FROM table GROUP BY a`
        group_by: Vec<Identifier>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// What to select in a query
pub enum SelectResultExpr {
    /// All columns in a table e.g. `SELECT * FROM table`
    ALL,
    /// A single expression e.g. `SELECT a FROM table`
    AliasedResultExpr(AliasedResultExpr),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// An expression with an alias e.g. `a + 1 AS b`
pub struct AliasedResultExpr {
    /// The expression e.g. `a + 1`, `COUNT(*)`, etc.
    pub expr: Box<Expression>,
    /// The alias e.g. `count` in `COUNT(*) AS count`
    pub alias: Identifier,
}

impl AliasedResultExpr {
    /// Create a new `AliasedResultExpr`
    #[must_use]
    pub fn new(expr: Expression, alias: Identifier) -> Self {
        Self {
            expr: Box::new(expr),
            alias,
        }
    }

    /// Try to get the identifier of the expression if it is a column
    /// Otherwise return None
    #[must_use]
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
        /// The qualified table Identifier
        table: Identifier,
        /// Namespace / schema for the table
        schema: Option<Identifier>,
    },
}

/// Binary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

    /// Comparison <
    LessThan,

    /// Comparison >
    GreaterThan,
}

/// Possible unary operators for simple expressions
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UnaryOperator {
    /// Logical inversion
    Not,
}

// Aggregation operators
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// Aggregation operators
pub enum AggregationOperator {
    /// Maximum
    Max,
    /// Minimum
    Min,
    /// Sum
    Sum,
    /// Count
    Count,
    /// Return the first value
    First,
}

impl Display for AggregationOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expression {
    /// Literal
    Literal(Literal),

    /// Column
    Column(Identifier),

    /// Unary operation
    Unary {
        /// The unary operator
        op: UnaryOperator,
        /// The expression to apply the operator to
        expr: Box<Expression>,
    },

    /// Binary operation
    Binary {
        /// The binary operator
        op: BinaryOperator,
        /// The left hand side of the operation
        left: Box<Expression>,
        /// The right hand side of the operation
        right: Box<Expression>,
    },

    /// * expression
    Wildcard,

    /// Aggregation operation
    Aggregation {
        /// The aggregation operator
        op: AggregationOperator,
        /// The expression to aggregate
        expr: Box<Expression>,
    },
}

impl Expression {
    /// Create a new `SUM()`
    #[must_use]
    pub fn sum(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Sum,
            expr: Box::new(self),
        })
    }

    /// Create a new `MAX()`
    #[must_use]
    pub fn max(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Max,
            expr: Box::new(self),
        })
    }

    /// Create a new `MIN()`
    #[must_use]
    pub fn min(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Min,
            expr: Box::new(self),
        })
    }

    /// Create a new `COUNT()`
    #[must_use]
    pub fn count(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::Count,
            expr: Box::new(self),
        })
    }

    /// Create a new `FIRST()`
    #[must_use]
    pub fn first(self) -> Box<Self> {
        Box::new(Expression::Aggregation {
            op: AggregationOperator::First,
            expr: Box::new(self),
        })
    }
    /// Create an `AliasedResultExpr` from an `Expression` using the provided alias.
    /// # Panics
    ///
    /// This function will panic if the provided `alias` cannot be parsed into an `Identifier`.
    /// It will also panic if `self` cannot be boxed.
    #[must_use]
    pub fn alias(self, alias: &str) -> AliasedResultExpr {
        AliasedResultExpr {
            expr: Box::new(self),
            alias: alias.parse().unwrap(),
        }
    }
}
impl core::ops::Add<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn add(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Add,
            left: self,
            right: rhs,
        })
    }
}
impl core::ops::Mul<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn mul(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Multiply,
            left: self,
            right: rhs,
        })
    }
}
impl core::ops::Div<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn div(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Division,
            left: self,
            right: rhs,
        })
    }
}
impl core::ops::Sub<Box<Expression>> for Box<Expression> {
    type Output = Box<Expression>;

    fn sub(self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary {
            op: BinaryOperator::Subtract,
            left: self,
            right: rhs,
        })
    }
}

/// `OrderBy`
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OrderBy {
    /// which column to order by
    pub expr: Identifier,
    /// in which direction to order
    pub direction: OrderByDirection,
}

/// `OrderByDirection` values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderByDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl Display for OrderByDirection {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    /// if `u64::MAX`, specify all rows
    pub number_rows: u64,

    /// number of rows to skip
    ///
    /// if 0, specify the first row as starting point
    /// if negative, specify the offset from the end
    /// (e.g. -1 is the last row, -2 is the second to last row, etc.)
    pub offset_value: i64,
}

/// Literal values
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub enum Literal {
    /// Boolean Literal
    Boolean(bool),
    /// i64 Literal
    BigInt(i64),
    /// i128 Literal
    Int128(i128),
    /// String Literal
    VarChar(String),
    /// Decimal Literal
    Decimal(BigDecimal),
    /// Timestamp Literal
    Timestamp(PoSQLTimestamp),
}

impl From<bool> for Literal {
    fn from(val: bool) -> Self {
        Literal::Boolean(val)
    }
}

/// TODO: add docs
macro_rules! impl_int_to_literal {
    ($tt:ty) => {
        impl From<$tt> for Literal {
            fn from(val: $tt) -> Self {
                Literal::BigInt(i64::from(val))
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

impl From<i128> for Literal {
    fn from(val: i128) -> Self {
        Literal::Int128(val)
    }
}

/// TODO: add docs
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

impl From<BigDecimal> for Literal {
    fn from(val: BigDecimal) -> Self {
        Literal::Decimal(val)
    }
}

impl From<PoSQLTimestamp> for Literal {
    fn from(time: PoSQLTimestamp) -> Self {
        Literal::Timestamp(time)
    }
}

/// Helper function to append an item to a vector
pub(crate) fn append<T>(list: Vec<T>, item: T) -> Vec<T> {
    let mut result = list;
    result.push(item);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_append_helper() {
        let list = vec![1, 2, 3];
        let result = append(list.clone(), 4);
        assert_eq!(result, vec![1, 2, 3, 4]);
        assert_eq!(list, vec![1, 2, 3]); // Original list unchanged
    }

    #[test]
    fn test_expression_operations() {
        // Test sum operation
        let expr = Expression::Column(Identifier::new("test_col"));
        let sum_expr = expr.sum();
        assert!(matches!(
            *sum_expr,
            Expression::Aggregation {
                op: AggregationOperator::Sum,
                expr: _
            }
        ));

        // Test max operation
        let expr = Expression::Column(Identifier::new("test_col"));
        let max_expr = expr.max();
        assert!(matches!(
            *max_expr,
            Expression::Aggregation {
                op: AggregationOperator::Max,
                expr: _
            }
        ));

        // Test min operation
        let expr = Expression::Column(Identifier::new("test_col"));
        let min_expr = expr.min();
        assert!(matches!(
            *min_expr,
            Expression::Aggregation {
                op: AggregationOperator::Min,
                expr: _
            }
        ));

        // Test count operation
        let expr = Expression::Column(Identifier::new("test_col"));
        let count_expr = expr.count();
        assert!(matches!(
            *count_expr,
            Expression::Aggregation {
                op: AggregationOperator::Count,
                expr: _
            }
        ));

        // Test first operation
        let expr = Expression::Column(Identifier::new("test_col"));
        let first_expr = expr.first();
        assert!(matches!(
            *first_expr,
            Expression::Aggregation {
                op: AggregationOperator::First,
                expr: _
            }
        ));
    }

    #[test]
    fn test_expression_alias() {
        let expr = Expression::Column(Identifier::new("test_col"));
        let aliased = expr.alias("alias_name");
        assert_eq!(aliased.alias.as_str(), "alias_name");
        assert!(matches!(*aliased.expr, Expression::Column(_)));
    }

    #[test]
    fn test_binary_operators() {
        let left = Box::new(Expression::Literal(Literal::BigInt(1)));
        let right = Box::new(Expression::Literal(Literal::BigInt(2)));

        // Test addition
        let add = left.clone() + right.clone();
        assert!(matches!(
            *add,
            Expression::Binary {
                op: BinaryOperator::Add,
                left: _,
                right: _
            }
        ));

        // Test multiplication
        let mul = left.clone() * right.clone();
        assert!(matches!(
            *mul,
            Expression::Binary {
                op: BinaryOperator::Multiply,
                left: _,
                right: _
            }
        ));

        // Test division
        let div = left.clone() / right.clone();
        assert!(matches!(
            *div,
            Expression::Binary {
                op: BinaryOperator::Division,
                left: _,
                right: _
            }
        ));

        // Test subtraction
        let sub = left - right;
        assert!(matches!(
            *sub,
            Expression::Binary {
                op: BinaryOperator::Subtract,
                left: _,
                right: _
            }
        ));
    }

    #[test]
    fn test_literal_conversions() {
        // Test bool conversion
        let bool_lit: Literal = true.into();
        assert!(matches!(bool_lit, Literal::Boolean(true)));

        // Test integer conversions
        let i8_lit: Literal = 42i8.into();
        assert!(matches!(i8_lit, Literal::BigInt(42)));

        let u8_lit: Literal = 42u8.into();
        assert!(matches!(u8_lit, Literal::BigInt(42)));

        let i16_lit: Literal = 42i16.into();
        assert!(matches!(i16_lit, Literal::BigInt(42)));

        let u16_lit: Literal = 42u16.into();
        assert!(matches!(u16_lit, Literal::BigInt(42)));

        let i32_lit: Literal = 42i32.into();
        assert!(matches!(i32_lit, Literal::BigInt(42)));

        let u32_lit: Literal = 42u32.into();
        assert!(matches!(u32_lit, Literal::BigInt(42)));

        let i64_lit: Literal = 42i64.into();
        assert!(matches!(i64_lit, Literal::BigInt(42)));

        // Test i128 conversion
        let i128_lit: Literal = 42i128.into();
        assert!(matches!(i128_lit, Literal::Int128(42)));

        // Test string conversions
        let str_lit: Literal = "test".into();
        assert!(matches!(str_lit, Literal::VarChar(_)));

        let string_lit: Literal = "test".to_string().into();
        assert!(matches!(string_lit, Literal::VarChar(_)));
    }

    #[test]
    fn test_aggregation_operator_display() {
        assert_eq!(AggregationOperator::Max.to_string(), "max");
        assert_eq!(AggregationOperator::Min.to_string(), "min");
        assert_eq!(AggregationOperator::Sum.to_string(), "sum");
        assert_eq!(AggregationOperator::Count.to_string(), "count");
        assert_eq!(AggregationOperator::First.to_string(), "first");
    }

    #[test]
    fn test_order_by_direction_display() {
        assert_eq!(OrderByDirection::Asc.to_string(), "asc");
        assert_eq!(OrderByDirection::Desc.to_string(), "desc");
    }

    #[test]
    fn test_aliased_result_expr_try_as_identifier() {
        let col_expr = Expression::Column(Identifier::new("test_col"));
        let aliased = AliasedResultExpr::new(col_expr, Identifier::new("alias"));
        assert!(aliased.try_as_identifier().is_some());

        let lit_expr = Expression::Literal(Literal::BigInt(42));
        let aliased = AliasedResultExpr::new(lit_expr, Identifier::new("alias"));
        assert!(aliased.try_as_identifier().is_none());
    }
}
