use polars::prelude::{col, lit, DataType, Expr, Literal, Series};

use crate::base::database::{INT128_PRECISION, INT128_SCALE};
use crate::sql::transform::{CompositionExpr, GroupByExpr, OrderByExprs, SelectExpr, SliceExpr};

use proofs_sql::intermediate_ast;
use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, BinaryOperator, Expression, OrderBy, ResultExpr, Slice,
};
use proofs_sql::Identifier;

/// A builder for `ResultExpr` nodes.
#[derive(Default)]
pub struct ResultExprBuilder {
    has_group_by: bool,
    composition: CompositionExpr,
}

impl ResultExprBuilder {
    /// Chain a new `OrderByExprs` to the current `ResultExpr`.
    pub fn add_order_by(mut self, by_exprs: Vec<OrderBy>) -> Self {
        if !by_exprs.is_empty() {
            self.composition.add(Box::new(OrderByExprs::new(by_exprs)));
        }
        self
    }

    /// Chain a new `GroupByExpr` to the current `ResultExpr`.
    pub fn add_group_by(
        mut self,
        by_exprs: Vec<(Identifier, Option<Identifier>)>,
        agg_exprs: Vec<proofs_sql::intermediate_ast::AliasedResultExpr>,
    ) -> Self {
        if by_exprs.is_empty() {
            return self;
        }

        self.has_group_by = true;

        // Prefix added to the group by columns not appearing in the select clause.
        // This hides the column from the final select result.
        const NON_RESULT_BY_EXPR_PREFIX: &str = "#$";

        let by_exprs = by_exprs
            .into_iter()
            .map(|(expr, alias)| {
                let default_alias = NON_RESULT_BY_EXPR_PREFIX.to_owned() + expr.as_str();
                let alias = alias
                    .as_ref()
                    .map(|v| v.as_str())
                    .unwrap_or(default_alias.as_str());
                col(expr.as_str()).alias(alias)
            })
            .collect();
        let agg_exprs = agg_exprs.into_iter().map(visit_aliased_expr).collect();

        self.composition
            .add(Box::new(GroupByExpr::new(by_exprs, agg_exprs)));

        self
    }

    /// Chain a new `SelectExpr` to the current `ResultExpr`.
    pub fn add_select(mut self, columns: Vec<AliasedResultExpr>) -> Self {
        assert!(!columns.is_empty());

        let columns = if self.has_group_by {
            // Group by modifies the result schema order and name so that
            // only aliases exist in the final lazy frame.
            //
            // Therefore, we need to re-map the select expression to reflect
            // the group by changes.
            //
            // TODO: check the following case `select 2 * A, max(B) from T group by A`
            columns
                .iter()
                .map(|expr| col(expr.alias.as_str()))
                .collect::<Vec<_>>()
        } else {
            columns
                .into_iter()
                .map(visit_aliased_expr)
                .collect::<Vec<_>>()
        };
        self.composition.add(Box::new(SelectExpr::new(columns)));
        self
    }

    /// Chain a new `SliceExpr` to the current `ResultExpr`.
    pub fn add_slice(mut self, slice: &Option<Slice>) -> Self {
        let (number_rows, offset_value) = match slice {
            Some(Slice {
                number_rows,
                offset_value,
            }) => (*number_rows, *offset_value),
            None => (u64::MAX, 0),
        };

        // we don't need to add a slice transformation if
        // we are not limiting or shifting the number of rows
        if number_rows != u64::MAX || offset_value != 0 {
            self.composition
                .add(Box::new(SliceExpr::new(number_rows, offset_value)));
        }
        self
    }

    /// Build a `ResultExpr` from the current state of the builder.
    pub fn build(self) -> crate::sql::transform::ResultExpr {
        crate::sql::transform::ResultExpr::new(Box::new(self.composition))
    }
}

fn visit_aliased_expr(aliased_expr: AliasedResultExpr) -> Expr {
    visit_result_expr(aliased_expr.expr).alias(aliased_expr.alias.as_str())
}

fn visit_result_expr(result_expr: ResultExpr) -> Expr {
    match result_expr {
        ResultExpr::Agg(agg_expr) => match agg_expr {
            AggExpr::Max(expr) => visit_expression(*expr).max(),
            AggExpr::Min(expr) => visit_expression(*expr).min(),
            AggExpr::Sum(expr) => visit_expression(*expr).sum(),
            AggExpr::Count(expr) => visit_expression(*expr).count(),
            AggExpr::CountALL => panic!("CountALL must be remapped to 'count(col_id)'"),
        },
        ResultExpr::NonAgg(expr) => visit_expression(*expr),
    }
}

fn visit_expression(expr: proofs_sql::intermediate_ast::Expression) -> Expr {
    match expr {
        Expression::Literal(literal) => match literal {
            intermediate_ast::Literal::Int128(value) => {
                let s = [value.to_string()].into_iter().collect::<Series>();
                s.lit().cast(DataType::Decimal(
                    Some(INT128_PRECISION),
                    Some(INT128_SCALE),
                ))
            }
            intermediate_ast::Literal::VarChar(value) => lit(value),
        },
        Expression::Column(identifier) => col(identifier.as_str()),
        Expression::Binary { op, left, right } => {
            let left = visit_expression(*left);
            let right = visit_expression(*right);

            match op {
                BinaryOperator::Add => left + right,
                BinaryOperator::Subtract => left - right,
                BinaryOperator::Multiply => left * right,
                _ => panic!("Operation not supported yet"),
            }
        }
        _ => panic!("Operation not supported"),
    }
}
