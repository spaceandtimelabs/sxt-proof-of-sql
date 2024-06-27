use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType, LiteralValue, TableRef},
    },
    sql::{
        ast::{
            AggregateExpr, AliasedProvableExprPlan, ColumnExpr, GroupByExpr, ProvableExprPlan,
            TableExpr,
        },
        parse::{ConversionError, ConversionResult, ProvableExprPlanBuilder, WhereExprBuilder},
    },
};
use proof_of_sql_parser::{
    intermediate_ast::{AggregationOperator, AliasedResultExpr, Expression, OrderBy, Slice},
    Identifier,
};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct QueryContext {
    in_agg_scope: bool,
    agg_counter: usize,
    slice_expr: Option<Slice>,
    col_ref_counter: usize,
    table: Option<TableRef>,
    in_result_scope: bool,
    has_visited_group_by: bool,
    order_by_exprs: Vec<OrderBy>,
    group_by_exprs: Vec<Identifier>,
    where_expr: Option<Box<Expression>>,
    result_column_set: HashSet<Identifier>,
    res_aliased_exprs: Vec<AliasedResultExpr>,
    column_mapping: HashMap<Identifier, ColumnRef>,
    first_result_col_out_agg_scope: Option<Identifier>,
}

impl QueryContext {
    pub fn set_table_ref(&mut self, table: TableRef) {
        assert!(self.table.is_none());
        self.table = Some(table);
    }

    pub fn get_table_ref(&self) -> &TableRef {
        self.table
            .as_ref()
            .expect("Table should already have been set")
    }

    pub fn set_where_expr(&mut self, where_expr: Option<Box<Expression>>) {
        self.where_expr = where_expr;
    }

    pub fn get_where_expr(&self) -> &Option<Box<Expression>> {
        &self.where_expr
    }

    pub fn set_slice_expr(&mut self, slice_expr: Option<Slice>) {
        self.slice_expr = slice_expr;
    }

    pub fn toggle_result_scope(&mut self) {
        self.in_result_scope = !self.in_result_scope;
    }

    pub fn is_in_result_scope(&self) -> bool {
        self.in_result_scope
    }

    pub fn set_in_agg_scope(&mut self, in_agg_scope: bool) -> ConversionResult<()> {
        if !in_agg_scope {
            assert!(
                self.in_agg_scope,
                "aggregation context needs to be set before exiting"
            );
            self.in_agg_scope = false;
            return Ok(());
        }

        if self.in_agg_scope {
            // TODO: Disable this once we support nested aggregations
            return Err(ConversionError::InvalidExpression(
                "nested aggregations are not supported".to_string(),
            ));
        }

        self.agg_counter += 1;
        self.in_agg_scope = true;

        Ok(())
    }

    fn is_in_agg_scope(&self) -> bool {
        self.in_agg_scope
    }

    pub fn push_column_ref(&mut self, column: Identifier, column_ref: ColumnRef) {
        self.col_ref_counter += 1;
        self.push_result_column_ref(column);
        self.column_mapping.insert(column, column_ref);
    }

    fn push_result_column_ref(&mut self, column: Identifier) {
        if self.is_in_result_scope() {
            self.result_column_set.insert(column);

            if !self.is_in_agg_scope() && self.first_result_col_out_agg_scope.is_none() {
                self.first_result_col_out_agg_scope = Some(column);
            }
        }
    }

    pub fn push_aliased_result_expr(&mut self, expr: AliasedResultExpr) -> ConversionResult<()> {
        assert!(&self.has_visited_group_by, "Group by must be visited first");
        self.res_aliased_exprs.push(expr);

        Ok(())
    }

    pub fn set_group_by_exprs(&mut self, exprs: Vec<Identifier>) {
        self.group_by_exprs = exprs;

        // Add the group by columns to the result column set
        // to ensure their integrity in the filter expression.
        for group_column in &self.group_by_exprs {
            self.result_column_set.insert(*group_column);
        }

        self.has_visited_group_by = true;
    }

    pub fn set_order_by_exprs(&mut self, order_by_exprs: Vec<OrderBy>) {
        self.order_by_exprs = order_by_exprs;
    }

    pub fn get_any_result_column_ref(&self) -> Option<(Identifier, ColumnType)> {
        // For tests to work we need to make it deterministic by sorting the columns
        // In the long run we simply need to let * be *
        // and get rid of this workaround altogether
        let mut columns = self.result_column_set.iter().collect::<Vec<_>>();
        columns.sort();
        columns.first().map(|c| {
            let column = self.column_mapping[c];
            (column.column_id(), *column.column_type())
        })
    }

    pub fn is_in_group_by_exprs(&self, column: &Identifier) -> ConversionResult<bool> {
        // Non-aggregated result column references must be included in the group by statement.
        if self.group_by_exprs.is_empty() || self.is_in_agg_scope() || !self.is_in_result_scope() {
            return Ok(false);
        }

        // Result column references outside aggregation must appear in the group by
        self.group_by_exprs
            .iter()
            .find(|group_column| *group_column == column)
            .map(|_| true)
            .ok_or(ConversionError::InvalidGroupByColumnRef(column.to_string()))
    }

    pub fn get_aliased_result_exprs(&self) -> ConversionResult<&[AliasedResultExpr]> {
        assert!(!self.res_aliased_exprs.is_empty(), "empty aliased exprs");

        // We need to check that each column alias is unique
        for col in &self.res_aliased_exprs {
            if self
                .res_aliased_exprs
                .iter()
                .map(|c| (c.alias == col.alias) as u64)
                .sum::<u64>()
                != 1
            {
                return Err(ConversionError::DuplicateResultAlias(col.alias.to_string()));
            }
        }

        // We cannot have column references outside aggregations when there is no group by expressions
        if self.group_by_exprs.is_empty()
            && self.agg_counter > 0
            && self.first_result_col_out_agg_scope.is_some()
        {
            return Err(ConversionError::InvalidGroupByColumnRef(
                self.first_result_col_out_agg_scope.unwrap().to_string(),
            ));
        }

        Ok(&self.res_aliased_exprs)
    }

    pub fn get_order_by_exprs(&self) -> ConversionResult<Vec<OrderBy>> {
        // Order by must reference only aliases in the result schema
        for by_expr in &self.order_by_exprs {
            self.res_aliased_exprs
                .iter()
                .find(|col| col.alias == by_expr.expr)
                .ok_or(ConversionError::InvalidOrderBy(
                    by_expr.expr.as_str().to_string(),
                ))?;
        }

        Ok(self.order_by_exprs.clone())
    }

    pub fn get_slice_expr(&self) -> &Option<Slice> {
        &self.slice_expr
    }

    pub fn get_group_by_exprs(&self) -> &[Identifier] {
        &self.group_by_exprs
    }

    pub fn get_result_column_set(&self) -> HashSet<Identifier> {
        self.result_column_set.clone()
    }

    pub fn get_column_mapping(&self) -> HashMap<Identifier, ColumnRef> {
        self.column_mapping.clone()
    }
}

/// Converts a `QueryContext` into a `Option<GroupByExpr>`.
///
/// We use Some if the query is provable and None if it is not
/// We error out if the query is wrong
impl<C: Commitment> TryFrom<&QueryContext> for Option<GroupByExpr<C>> {
    type Error = ConversionError;

    fn try_from(value: &QueryContext) -> Result<Option<GroupByExpr<C>>, Self::Error> {
        // Currently if there is no where clause, we can't prove the query
        if value.where_expr.is_none() {
            return Ok(None);
        }
        let where_clause = WhereExprBuilder::new(&value.column_mapping)
            .build(value.where_expr.clone())?
            .unwrap_or_else(|| ProvableExprPlan::new_literal(LiteralValue::Boolean(true)));
        let table = value.table.map(|table_ref| TableExpr { table_ref }).ok_or(
            ConversionError::InvalidExpression("QueryContext has no table_ref".to_owned()),
        )?;
        let resource_id = table.table_ref.resource_id();
        let group_by_exprs = value
            .group_by_exprs
            .iter()
            .map(|expr| -> Result<ColumnExpr<C>, ConversionError> {
                value
                    .column_mapping
                    .get(expr)
                    .ok_or(ConversionError::MissingColumn(
                        Box::new(*expr),
                        Box::new(resource_id),
                    ))
                    .map(|column_ref| ColumnExpr::<C>::new(*column_ref))
            })
            .collect::<Result<Vec<ColumnExpr<C>>, ConversionError>>()?;
        // For a query to be provable the result columns must be of one of three kinds below:
        // 1. Expressions exclusively consisting of group by columns
        // 2. Sum(expr) expressions with an alias
        // 3. count(expr) with an alias
        let opt_res_expr_plans = value
            .res_aliased_exprs
            .iter()
            .map(|res| {
                let res_provable_expr_plan =
                    ProvableExprPlanBuilder::new(&value.column_mapping).build(&res.expr);
                res_provable_expr_plan
                    .ok()
                    .map(|provable_expr_plan| AliasedProvableExprPlan {
                        alias: res.alias,
                        expr: provable_expr_plan,
                    })
            })
            .collect::<Option<Vec<AliasedProvableExprPlan<C>>>>();
        if opt_res_expr_plans.is_none() {
            return Ok(None);
        }
        let res_expr_plans = opt_res_expr_plans.expect("the none case was just checked");
        let sum_exprs = res_expr_plans
            .iter()
            .filter_map(|res| {
                if let ProvableExprPlan::Aggregate(AggregateExpr {
                    op: AggregationOperator::Sum,
                    expr,
                }) = &res.expr
                {
                    Some(AliasedProvableExprPlan {
                        alias: res.alias,
                        expr: *expr.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<AliasedProvableExprPlan<C>>>();
        let count_exprs = res_expr_plans
            .iter()
            .filter_map(|res| {
                if let ProvableExprPlan::Aggregate(AggregateExpr {
                    op: AggregationOperator::Count,
                    expr,
                }) = &res.expr
                {
                    Some(AliasedProvableExprPlan {
                        alias: res.alias,
                        expr: *expr.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<AliasedProvableExprPlan<C>>>();
        let res_exprs = res_expr_plans
            .iter()
            .filter_map(|res| {
                if let ProvableExprPlan::Aggregate(_expr) = &res.expr {
                    None
                } else {
                    Some(AliasedProvableExprPlan {
                        alias: res.alias,
                        expr: res.expr.clone(),
                    })
                }
            })
            .collect::<Vec<AliasedProvableExprPlan<C>>>();
        Ok(Some(GroupByExpr::new(
            group_by_exprs,
            res_exprs,
            sum_exprs,
            count_exprs,
            table,
            where_clause,
        )))
    }
}
