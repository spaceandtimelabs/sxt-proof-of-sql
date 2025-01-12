use proof_of_sql_parser::{intermediate_ast::Literal, sqlparser::SqlAliasedResultExpr};
use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArg, FunctionArgExpr, Ident, ObjectName, UnaryOperator,
};

/// Compute the sum of an expression
#[must_use]
pub fn sum(expr: Expr) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new("SUM")]),
        args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(*Box::new(expr)))],
        filter: None,
        null_treatment: None,
        over: None,
        distinct: false,
        special: false,
        order_by: vec![],
    })
}

/// Get column from name
///
/// # Panics
///
/// This function will panic if the name cannot be parsed into a valid column expression as valid [Identifier]s.
#[must_use]
pub fn col(name: &str) -> Expr {
    Expr::Identifier(name.into())
}

/// Compute the maximum of an expression
#[must_use]
pub fn max(expr: Expr) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new("MAX")]),
        args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(*Box::new(expr)))],
        filter: None,
        null_treatment: None,
        over: None,
        distinct: false,
        special: false,
        order_by: vec![],
    })
}

/// Construct a new `Expr` A + B
#[must_use]
pub fn add(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Plus,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Construct a new `Expr` A - B
#[must_use]
pub fn sub(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op: BinaryOperator::Minus,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Get literal from value
pub fn lit<L>(literal: L) -> Expr
where
    L: Into<Literal>,
{
    Expr::from(literal.into())
}

/// Count the amount of non-null entries of an expression
#[must_use]
pub fn count(expr: Expr) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new("COUNT")]),
        args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(*Box::new(expr)))],
        filter: None,
        null_treatment: None,
        over: None,
        distinct: false,
        special: false,
        order_by: vec![],
    })
}

/// Count the rows
#[must_use]
pub fn count_all() -> Expr {
    count(Expr::Wildcard)
}

/// Construct a new `Expr` representing A * B
#[must_use]
pub fn mul(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::Multiply,
        right: Box::new(right),
    }
}

/// Compute the minimum of an expression
#[must_use]
pub fn min(expr: Expr) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new("MIN")]),
        args: vec![FunctionArg::Unnamed(FunctionArgExpr::Expr(*Box::new(expr)))],
        filter: None,
        null_treatment: None,
        over: None,
        distinct: false,
        special: false,
        order_by: vec![],
    })
}

/// Construct a new `Expr` for NOT P
#[must_use]
pub fn not(expr: Expr) -> Expr {
    Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr: Box::new(expr),
    }
}

/// Construct a new `Expr` for A >= B
#[must_use]
pub fn ge(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::GtEq,
        right: Box::new(right),
    }
}

/// Construct a new `Expr` for A == B
#[must_use]
pub fn equal(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::Eq,
        right: Box::new(right),
    }
}

/// Construct a new `Expr` for P OR Q
#[must_use]
pub fn or(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::Or,
        right: Box::new(right),
    }
}

/// An expression with an alias, i.e., EXPR AS ALIAS
///
/// # Panics
///
/// This function will panic if the `alias` cannot be parsed as a valid [Identifier].
pub fn aliased_expr(expr: Expr, alias: &str) -> SqlAliasedResultExpr {
    SqlAliasedResultExpr {
        expr: Box::new(expr),
        alias: Ident::new(alias),
    }
}
