use super::{polars_arithmetic::SafeDivision, polars_conversions::LiteralConversion};
use polars::prelude::{col, Expr};
use proof_of_sql_parser::intermediate_ast::*;
pub(crate) trait ToPolarsExpr {
    fn to_polars_expr(&self) -> Expr;
}
#[cfg(test)]
impl ToPolarsExpr for Expr {
    fn to_polars_expr(&self) -> Expr {
        self.clone()
    }
}
#[cfg(test)]
impl<T: ToPolarsExpr> ToPolarsExpr for Box<T> {
    fn to_polars_expr(&self) -> Expr {
        self.as_ref().to_polars_expr()
    }
}
impl ToPolarsExpr for AliasedResultExpr {
    fn to_polars_expr(&self) -> Expr {
        self.expr.to_polars_expr().alias(self.alias.as_str())
    }
}
impl ToPolarsExpr for Expression {
    fn to_polars_expr(&self) -> Expr {
        match self {
            Expression::Literal(literal) => match literal {
                Literal::Boolean(value) => value.to_lit(),
                Literal::BigInt(value) => value.to_lit(),
                Literal::Int128(value) => value.to_lit(),
                Literal::VarChar(_) => panic!("Expression not supported"),
                Literal::Decimal(_) => todo!(),
                Literal::Timestamp(_) => panic!("Expression not supported"),
            },
            Expression::Column(identifier) => col(identifier.as_str()),
            Expression::Binary { op, left, right } => {
                let left = left.to_polars_expr();
                let right = right.to_polars_expr();
                match op {
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Division => left.checked_div(right),
                    _ => panic!("Operation not supported yet"),
                }
            }
            Expression::Aggregation { op, expr } => {
                let expr = expr.to_polars_expr();
                match op {
                    AggregationOperator::Count => expr.count(),
                    AggregationOperator::Sum => expr.sum(),
                    AggregationOperator::Min => expr.min(),
                    AggregationOperator::Max => expr.max(),
                    AggregationOperator::First => expr.first(),
                }
            }
            _ => panic!("Operation not supported"),
        }
    }
}
