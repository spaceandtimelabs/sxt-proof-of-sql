use crate::sql::transform::ResultExpr;
use crate::sql::transform::{CompositionExpr, GroupByExpr, OrderByExprs, SelectExpr, SliceExpr};
use proofs_sql::intermediate_ast::{AggExpr, OrderBy, ResultColumn};
use proofs_sql::Identifier;

/// A builder for `ResultExpr` nodes.
#[derive(Default)]
pub struct ResultExprBuilder {
    composition: CompositionExpr,
}

impl ResultExprBuilder {
    /// Chain a new `OrderByExprs` to the current `ResultExpr`.
    pub fn add_order_by(&mut self, by_exprs: Vec<OrderBy>) {
        if !by_exprs.is_empty() {
            self.composition.add(Box::new(OrderByExprs::new(by_exprs)));
        }
    }

    pub fn add_group_by(
        &mut self,
        by_exprs: Vec<(Identifier, Option<Identifier>)>,
        agg_exprs: Vec<AggExpr>,
    ) {
        if !by_exprs.is_empty() {
            self.composition
                .add(Box::new(GroupByExpr::new(by_exprs, agg_exprs)));
        }
    }

    /// Chain a new `SelectExpr` to the current `ResultExpr`.
    pub fn add_select(&mut self, columns: Vec<ResultColumn>) {
        assert!(!columns.is_empty());
        self.composition.add(Box::new(SelectExpr::new(columns)));
    }

    /// Chain a new `SliceExpr` to the current `ResultExpr`.
    pub fn add_slice(&mut self, number_rows: u64, offset_value: i64) {
        // we don't need to add a slice transformation if
        // we are not limiting or shifting the number of rows
        if number_rows == u64::MAX && offset_value == 0 {
            return;
        }

        self.composition
            .add(Box::new(SliceExpr::new(number_rows, offset_value)));
    }

    /// Build a `ResultExpr` from the current state of the builder.
    pub fn build(self) -> ResultExpr {
        ResultExpr::new_with_transformation(Box::new(self.composition))
    }
}
