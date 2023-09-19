use polars::prelude::{col, Expr};

use crate::sql::transform::{
    CompositionExpr, GroupByExpr, LiteralConversion, OrderByExprs, SelectExpr, SliceExpr,
};

use proofs_sql::intermediate_ast;
use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, BinaryOperator, Expression, OrderBy, ResultExpr, Slice,
};
use proofs_sql::Identifier;

/// A builder for `ResultExpr` nodes.
#[derive(Default)]
pub struct ResultExprBuilder {
    composition: CompositionExpr,
}

impl ResultExprBuilder {
    /// Chain a new `GroupByExpr` to the current `ResultExpr`.
    pub fn add_group_by_exprs(
        mut self,
        by_exprs: &[Identifier],
        aliased_exprs: &[AliasedResultExpr],
    ) -> Self {
        if by_exprs.is_empty() {
            return self;
        }

        let polars_by_exprs: Vec<_> = by_exprs.iter().map(|id| col(id)).collect();
        let polars_agg_exprs = aliased_exprs
            .iter()
            .map(|expr| visit_aliased_expr(expr, by_exprs))
            .collect();

        self.composition.add(Box::new(GroupByExpr::new(
            polars_by_exprs,
            polars_agg_exprs,
        )));

        self
    }

    /// Chain a new `SelectExpr` to the current `ResultExpr`.
    pub fn add_select_exprs(mut self, aliased_exprs: &[AliasedResultExpr]) -> Self {
        assert!(!aliased_exprs.is_empty());

        let polars_exprs = aliased_exprs
            .iter()
            .map(|aliased_expr| {
                if !self.composition.is_empty() {
                    // The only transformation before a select is a group by.
                    // GROUP BY modifies the schema, so we need to
                    // update the code to reflect the changes.
                    col(aliased_expr.alias.as_str())
                } else {
                    visit_aliased_expr(aliased_expr, &[])
                }
            })
            .collect();

        self.composition
            .add(Box::new(SelectExpr::new(polars_exprs)));
        self
    }

    /// Chain a new `OrderByExprs` to the current `ResultExpr`.
    pub fn add_order_by_exprs(mut self, by_exprs: Vec<OrderBy>) -> Self {
        if !by_exprs.is_empty() {
            self.composition.add(Box::new(OrderByExprs::new(by_exprs)));
        }
        self
    }

    /// Chain a new `SliceExpr` to the current `ResultExpr`.
    pub fn add_slice_expr(mut self, slice: &Option<Slice>) -> Self {
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

fn visit_aliased_expr(aliased_expr: &AliasedResultExpr, group_by_exprs: &[Identifier]) -> Expr {
    let expr = match &aliased_expr.expr {
        ResultExpr::Agg(agg_expr) => match agg_expr {
            AggExpr::Max(expr) => visit_expr(expr).max(),
            AggExpr::Min(expr) => visit_expr(expr).min(),
            AggExpr::Sum(expr) => visit_expr(expr).sum(),
            AggExpr::Count(expr) => visit_expr(expr).count(),
            AggExpr::CountALL => col(group_by_exprs.iter().next().unwrap()).count(),
        },
        ResultExpr::NonAgg(expr) => {
            let expr = visit_expr(expr);
            if !group_by_exprs.is_empty() {
                // Transforming the group by result columns into an expression is necessary
                // to prevent Polars from returning lists for aggregation results.
                expr.first()
            } else {
                expr
            }
        }
    };

    expr.alias(aliased_expr.alias.as_str())
}

fn visit_expr(expr: &Expression) -> Expr {
    match expr {
        Expression::Literal(literal) => match literal {
            intermediate_ast::Literal::Int128(value) => value.to_lit(),
            intermediate_ast::Literal::VarChar(_) => panic!("Not supported yet"),
        },
        Expression::Column(identifier) => col(identifier.as_str()),
        Expression::Binary { op, left, right } => {
            let left = visit_expr(left);
            let right = visit_expr(right);

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
