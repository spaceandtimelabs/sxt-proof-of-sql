use crate::sql::transform::ResultExpr;
use crate::sql::transform::{CompositionExpr, OrderByExprs};
use proofs_sql::intermediate_ast::OrderBy;

/// A builder for `ResultExpr` nodes.
#[derive(Default)]
pub struct ResultExprBuilder {
    composition: CompositionExpr,
}

impl ResultExprBuilder {
    /// Chain a new `OrderByExprs` to the current `ResultExpr`.
    pub fn order_by(mut self, by_exprs: Vec<OrderBy>) -> Self {
        if by_exprs.is_empty() {
            return self;
        }

        let order_expr = OrderByExprs::new(by_exprs);
        self.composition.add(Box::new(order_expr));

        self
    }

    /// Build a `ResultExpr` from the current state of the builder.
    pub fn build(self) -> ResultExpr {
        if self.composition.is_empty() {
            return ResultExpr::default();
        }

        ResultExpr::new(Box::new(self.composition))
    }
}
