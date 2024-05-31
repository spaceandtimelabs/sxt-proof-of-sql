use crate::sql::transform::{CompositionExpr, GroupByExpr, OrderByExprs, SelectExpr, SliceExpr};
use proofs_sql::{
    intermediate_ast::{AliasedResultExpr, Expression, OrderBy, Slice},
    Identifier,
};

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
        self.composition
            .add(Box::new(GroupByExpr::new(by_exprs, aliased_exprs)));
        self
    }

    /// Chain a new `SelectExpr` to the current `ResultExpr`.
    pub fn add_select_exprs(mut self, aliased_exprs: &[AliasedResultExpr]) -> Self {
        assert!(!aliased_exprs.is_empty());
        if !self.composition.is_empty() {
            // The only transformation before a select is a group by.
            // GROUP BY modifies the schema, so we need to
            // update the code to reflect the changes.
            let exprs: Vec<_> = aliased_exprs
                .iter()
                .map(|aliased_expr| Expression::Column(aliased_expr.alias))
                .collect();
            self.composition
                .add(Box::new(SelectExpr::new_from_expressions(&exprs)));
        } else {
            self.composition
                .add(Box::new(SelectExpr::new_from_aliased_result_exprs(
                    aliased_exprs,
                )));
        }
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
