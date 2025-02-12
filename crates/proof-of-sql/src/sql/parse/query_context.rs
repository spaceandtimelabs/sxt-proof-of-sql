use crate::{
    base::{
        database::{ColumnRef, LiteralValue, TableRef},
        map::{IndexMap, IndexSet},
    },
    sql::{
        parse::{ConversionError, ConversionResult, DynProofExprBuilder, WhereExprBuilder},
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
        proof_plans::GroupByExec,
    },
};
use alloc::{borrow::ToOwned, boxed::Box, string::ToString, vec::Vec};
use proof_of_sql_parser::intermediate_ast::{
    AggregationOperator, AliasedResultExpr, Expression, OrderBy, Slice,
};
use sqlparser::ast::Ident;

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
    group_by_exprs: Vec<Ident>,
    where_expr: Option<Box<Expression>>,
    result_column_set: IndexSet<Ident>,
    res_aliased_exprs: Vec<AliasedResultExpr>,
    column_mapping: IndexMap<Ident, ColumnRef>,
    first_result_col_out_agg_scope: Option<Ident>,
}

impl QueryContext {
    #[allow(clippy::missing_panics_doc)]
    pub fn set_table_ref(&mut self, table: TableRef) {
        assert!(self.table.is_none());
        self.table = Some(table);
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn get_table_ref(&self) -> &TableRef {
        self.table
            .as_ref()
            .expect("Table should already have been set")
    }

    pub fn set_where_expr(&mut self, where_expr: Option<Box<Expression>>) {
        self.where_expr = where_expr;
    }

    #[allow(clippy::ref_option)]
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

    #[allow(clippy::missing_panics_doc)]
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
            return Err(ConversionError::InvalidExpression {
                expression: "nested aggregations are not supported".to_string(),
            });
        }

        self.agg_counter += 1;
        self.in_agg_scope = true;

        Ok(())
    }

    fn is_in_agg_scope(&self) -> bool {
        self.in_agg_scope
    }

    /// TODO: add docs
    pub(crate) fn has_agg(&self) -> bool {
        self.agg_counter > 0 || !self.group_by_exprs.is_empty()
    }

    pub fn push_column_ref(&mut self, column: Ident, column_ref: ColumnRef) {
        self.col_ref_counter += 1;
        self.push_result_column_ref(column.clone());
        self.column_mapping.insert(column, column_ref);
    }

    fn push_result_column_ref(&mut self, column: Ident) {
        if self.is_in_result_scope() {
            self.result_column_set.insert(column.clone());

            if !self.is_in_agg_scope() && self.first_result_col_out_agg_scope.is_none() {
                self.first_result_col_out_agg_scope = Some(column);
            }
        }
    }

    #[allow(clippy::missing_panics_doc, clippy::unnecessary_wraps)]
    pub fn push_aliased_result_expr(&mut self, expr: AliasedResultExpr) -> ConversionResult<()> {
        assert!(&self.has_visited_group_by, "Group by must be visited first");
        self.res_aliased_exprs.push(expr);

        Ok(())
    }

    pub fn set_group_by_exprs(&mut self, exprs: Vec<Ident>) {
        self.group_by_exprs = exprs;

        // Add the group by columns to the result column set
        // to ensure their integrity in the filter expression.
        for group_column in &self.group_by_exprs {
            self.result_column_set.insert(group_column.clone());
        }

        self.has_visited_group_by = true;
    }

    pub fn set_order_by_exprs(&mut self, order_by_exprs: Vec<OrderBy>) {
        self.order_by_exprs = order_by_exprs;
    }

    pub fn is_in_group_by_exprs(&self, column: &Ident) -> ConversionResult<bool> {
        // Non-aggregated result column references must be included in the group by statement.
        if self.group_by_exprs.is_empty() || self.is_in_agg_scope() || !self.is_in_result_scope() {
            return Ok(false);
        }

        // Result column references outside aggregation must appear in the group by
        self.group_by_exprs
            .iter()
            .find(|group_column| *group_column == column)
            .map(|_| true)
            .ok_or(ConversionError::InvalidGroupByColumnRef {
                column: column.to_string(),
            })
    }

    /// # Panics
    ///
    /// Will panic if:
    /// - `self.res_aliased_exprs` is empty, triggering the assertion `assert!(!self.res_aliased_exprs.is_empty(), "empty aliased exprs")`.
    pub fn get_aliased_result_exprs(&self) -> ConversionResult<&[AliasedResultExpr]> {
        assert!(!self.res_aliased_exprs.is_empty(), "empty aliased exprs");

        // We need to check that each column alias is unique
        for col in &self.res_aliased_exprs {
            if self
                .res_aliased_exprs
                .iter()
                .map(|c| u64::from(c.alias == col.alias))
                .sum::<u64>()
                != 1
            {
                return Err(ConversionError::DuplicateResultAlias {
                    alias: col.alias.to_string(),
                });
            }
        }

        // We cannot have column references outside aggregations when there is no group by expressions
        if self.group_by_exprs.is_empty()
            && self.agg_counter > 0
            && self.first_result_col_out_agg_scope.is_some()
        {
            return Err(ConversionError::InvalidGroupByColumnRef {
                column: self
                    .first_result_col_out_agg_scope
                    .as_ref()
                    .unwrap()
                    .to_string(),
            });
        }

        Ok(&self.res_aliased_exprs)
    }

    pub fn get_order_by_exprs(&self) -> ConversionResult<Vec<OrderBy>> {
        // Order by must reference only aliases in the result schema
        for by_expr in &self.order_by_exprs {
            self.res_aliased_exprs
                .iter()
                .find(|col| col.alias == by_expr.expr)
                .ok_or(ConversionError::InvalidOrderBy {
                    alias: by_expr.expr.as_str().to_string(),
                })?;
        }

        Ok(self.order_by_exprs.clone())
    }

    #[allow(clippy::ref_option)]
    pub fn get_slice_expr(&self) -> &Option<Slice> {
        &self.slice_expr
    }

    pub fn get_group_by_exprs(&self) -> &[Ident] {
        &self.group_by_exprs
    }

    pub fn get_result_column_set(&self) -> IndexSet<Ident> {
        self.result_column_set.clone()
    }

    pub fn get_column_mapping(&self) -> IndexMap<Ident, ColumnRef> {
        self.column_mapping.clone()
    }
}

/// Converts a `QueryContext` into an `Option<GroupByExec>`.
///
/// We use Some if the query is provable and None if it is not
/// We error out if the query is wrong
impl TryFrom<&QueryContext> for Option<GroupByExec> {
    type Error = ConversionError;

    fn try_from(value: &QueryContext) -> Result<Option<GroupByExec>, Self::Error> {
        let where_clause = WhereExprBuilder::new(&value.column_mapping)
            .build(value.where_expr.clone())?
            .unwrap_or_else(|| DynProofExpr::new_literal(LiteralValue::Boolean(true)));
        let table = value
            .table
            .as_ref()
            .map(|table_ref| TableExpr {
                table_ref: table_ref.clone(),
            })
            .ok_or(ConversionError::InvalidExpression {
                expression: "QueryContext has no table_ref".to_owned(),
            })?;

        let group_by_exprs = value
            .group_by_exprs
            .iter()
            .map(|expr| -> Result<ColumnExpr, ConversionError> {
                value
                    .column_mapping
                    .get(expr)
                    .ok_or_else(|| ConversionError::MissingColumn {
                        identifier: Box::new(expr.clone()),
                        table_ref: table.table_ref.clone(),
                    })
                    .map(|column_ref| ColumnExpr::new(column_ref.clone()))
            })
            .collect::<Result<Vec<ColumnExpr>, ConversionError>>()?;
        // For a query to be provable the result columns must be of one of three kinds below:
        // 1. Group by columns (it is mandatory to have all of them in the correct order)
        // 2. Sum(expr) expressions (it is optional to have any)
        // 3. count(*) with an alias (it is mandatory to have one and only one)
        let num_group_by_columns = group_by_exprs.len();
        let num_result_columns = value.res_aliased_exprs.len();
        if num_result_columns < num_group_by_columns + 1 {
            return Ok(None);
        }
        let res_group_by_columns = &value.res_aliased_exprs[..num_group_by_columns].to_vec();
        let sum_expr_columns =
            &value.res_aliased_exprs[num_group_by_columns..num_result_columns - 1].to_vec();
        // Check group by columns
        let group_by_compliance = value
            .group_by_exprs
            .iter()
            .zip(res_group_by_columns.iter())
            .all(|(ident, res)| {
                if let Expression::Column(res_ident) = *res.expr {
                    Ident::from(res_ident) == *ident
                } else {
                    false
                }
            });

        // Check sums
        let sum_expr = sum_expr_columns
            .iter()
            .map(|res| {
                if let Expression::Aggregation {
                    op: AggregationOperator::Sum,
                    ..
                } = (*res.expr).clone()
                {
                    let res_dyn_proof_expr =
                        DynProofExprBuilder::new(&value.column_mapping).build(&res.expr);
                    res_dyn_proof_expr
                        .ok()
                        .map(|dyn_proof_expr| AliasedDynProofExpr {
                            alias: res.alias.into(),
                            expr: dyn_proof_expr,
                        })
                } else {
                    None
                }
            })
            .collect::<Option<Vec<AliasedDynProofExpr>>>();

        // Check count(*)
        let count_column = &value.res_aliased_exprs[num_result_columns - 1];
        let count_column_compliant = matches!(
            *count_column.expr,
            Expression::Aggregation {
                op: AggregationOperator::Count,
                ..
            }
        );

        if !group_by_compliance || sum_expr.is_none() || !count_column_compliant {
            return Ok(None);
        }
        Ok(Some(GroupByExec::new(
            group_by_exprs,
            sum_expr.expect("the none case was just checked"),
            count_column.alias.into(),
            table,
            where_clause,
        )))
    }
}
