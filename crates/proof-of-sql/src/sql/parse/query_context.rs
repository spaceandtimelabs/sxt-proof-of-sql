use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
    },
    sql::{
        ast::{ColumnExpr, GroupByExpr, ProvableExprPlan, TableExpr},
        parse::{ConversionError, ConversionResult, WhereExprBuilder},
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
    group_by_exprs: Vec<AliasedResultExpr>,
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
            return Err(ConversionError::InvalidExpression(
                "nested aggregations are invalid".to_string(),
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
        for aliased_group_by_expr in &self.group_by_exprs {
            self.result_column_set.insert(*aliased_group_by_expr.alias);
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
        let aliases = self
            .group_by_exprs
            .iter()
            .map(|aliased_expr| aliased_expr.alias)
            .collect::<Vec<_>>();
        aliases.as_slice()
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
            .map(|aliased_expr| -> Result<AliasedProvableExprPlan<C>, ConversionError> {
                let enriched_expr = EnrichedExpr::new(aliased_expr, value.column_mapping.clone());
                enriched_expr.get_provable_expr_plan()
            })
            .collect::<Result<Vec<AliasedProvableExprPlan<C>>, ConversionError>>()?;
        // Allowed result expressions
        // 1. SUM(expr) for arbitrary provable expressions expr
        // 2. COUNT(expr) for arbitrary provable expressions expr (which until NULL is introduced is basically count(1))
        // 3. expressions with group by aliases only
        // Look for count_column if exists
        let aliased_provable_exprs = value
            .res_aliased_exprs
            .iter()
            .map(|aliased_expr| -> Result<AliasedProvableExprPlan<C>, ConversionError> {
                let enriched_expr = EnrichedExpr::new(aliased_expr, value.column_mapping.clone());
                enriched_expr.get_provable_expr_plan()
            })
            .collect::<Result<Vec<AliasedProvableExprPlan<C>>, ConversionError>>()?;
        let mut count_columns = vec![];
        let mut sum_exprs = vec![];
        aliased_provable_exprs.iter().for_each(|aliased_expr| {
            match aliased_expr {
                AliasedProvableExprPlan::AggregateFunction(agg) => {
                    match agg.op {
                        AggregationOperator::Count => {
                            //TODO: When we accept NULL we need to keep track of what we are counting
                            count_columns.push(agg.alias);
                        }
                        AggregationOperator::Sum => {
                            sum_exprs.push(agg.clone());
                        }
                        _ => {}
                    }
                }
                AliasedProvableExprPlan::Column(_) => {}
            }
        });
        Ok(Some(GroupByExpr::new(
            group_by_exprs,
            sum_expr.expect("the none case was just checked"),
            count_column.alias,
            table,
            where_clause,
        )))
    }
}
