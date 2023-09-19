use std::collections::{HashMap, HashSet};

use crate::base::database::{ColumnRef, TableRef};
use crate::sql::parse::{ConversionError, ConversionResult};

use proofs_sql::intermediate_ast::{AliasedResultExpr, Expression, OrderBy, Slice};
use proofs_sql::Identifier;

#[derive(Default)]
pub struct QueryContext {
    slice: Option<Slice>,
    table: Option<TableRef>,
    col_ref_counter: usize,
    order_by_exprs: Vec<OrderBy>,
    agg_result_exprs: Vec<usize>,
    fixed_col_ref_counter: usize,
    non_agg_result_exprs: Vec<usize>,
    group_by_exprs: Vec<Identifier>,
    where_expr: Option<Box<Expression>>,
    result_column_set: HashSet<Identifier>,
    res_aliased_exprs: Vec<AliasedResultExpr>,
    column_mapping: HashMap<Identifier, ColumnRef>,
}

impl QueryContext {
    pub fn set_table(&mut self, table: TableRef) {
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

    pub fn set_slice(&mut self, slice: Option<Slice>) {
        self.slice = slice;
    }

    pub fn push_column_ref(&mut self, column: Identifier, column_ref: ColumnRef) {
        self.col_ref_counter += 1;
        self.column_mapping.insert(column, column_ref);
    }

    pub fn push_schema_column(&mut self, column: AliasedResultExpr, is_agg: bool) {
        self.res_aliased_exprs.push(column.clone());

        if is_agg {
            self.agg_result_exprs.push(self.res_aliased_exprs.len() - 1);
        } else {
            self.non_agg_result_exprs
                .push(self.res_aliased_exprs.len() - 1);
        }
    }

    pub fn push_result_column_reference(&mut self, column: Identifier) {
        self.result_column_set.insert(column);
    }

    pub fn push_group_by(&mut self, column: Identifier) {
        self.group_by_exprs.push(column);
    }

    pub fn push_order_by_exprs(&mut self, order_by_exprs: OrderBy) {
        self.order_by_exprs.push(order_by_exprs);
    }

    pub fn fix_columns_counter(&mut self) {
        self.fixed_col_ref_counter = self.col_ref_counter;
    }

    pub fn validate_columns_counter(&mut self) -> ConversionResult<()> {
        if self.col_ref_counter == self.fixed_col_ref_counter {
            return Err(ConversionError::InvalidExpression(
                "at least one column must be referenced in the result expression".to_string(),
            ));
        }
        self.fix_columns_counter();
        Ok(())
    }

    pub fn get_aliased_result_exprs(&self) -> ConversionResult<&[AliasedResultExpr]> {
        assert!(
            !self.res_aliased_exprs.is_empty(),
            "Result schema must not be empty"
        );

        // We need to check that each column alias is unique
        for col in &self.res_aliased_exprs {
            if self
                .res_aliased_exprs
                .iter()
                .map(|c| (c.alias == col.alias) as u64)
                .sum::<u64>()
                != 1
            {
                return Err(ConversionError::DuplicateColumnAlias(col.alias.to_string()));
            }
        }

        Ok(&self.res_aliased_exprs)
    }

    pub fn get_order_by_exprs(&self) -> ConversionResult<Vec<OrderBy>> {
        // order by must reference only aliases in the result schema
        for by_expr in &self.order_by_exprs {
            self.res_aliased_exprs
                .iter()
                .find(|col| col.alias == by_expr.expr)
                .ok_or(ConversionError::InvalidOrderByError(
                    by_expr.expr.as_str().to_string(),
                ))?;
        }

        Ok(self.order_by_exprs.clone())
    }

    pub fn get_slice_expr(&self) -> &Option<Slice> {
        &self.slice
    }

    pub fn get_group_by_exprs(&self) -> ConversionResult<&[Identifier]> {
        if self.group_by_exprs.is_empty() {
            if !self.agg_result_exprs.is_empty() {
                // We can't aggregate without specifying a group by column yet
                return Err(ConversionError::MissingGroupByError);
            }

            return Ok(&[]);
        }

        // We need to add the group by columns that are part of the result schema.
        for col_idx in self.non_agg_result_exprs.iter() {
            let col = &self.res_aliased_exprs[*col_idx];
            let col_id: Identifier = *col
                .try_as_identifier()
                .ok_or(ConversionError::InvalidGroupByResultColumnError)?;

            // We need to check that each non aggregated result column
            // is referenced in the group by clause.
            if !self
                .group_by_exprs
                .iter()
                .any(|group_by| *group_by == col_id)
            {
                return Err(ConversionError::InvalidGroupByResultColumnError);
            }
        }

        Ok(&self.group_by_exprs)
    }

    pub fn get_result_column_set(&self) -> HashSet<Identifier> {
        self.result_column_set.clone()
    }

    pub fn get_column_mapping(&self) -> HashMap<Identifier, ColumnRef> {
        self.column_mapping.clone()
    }
}
