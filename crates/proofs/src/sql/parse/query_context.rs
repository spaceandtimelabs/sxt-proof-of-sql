use std::collections::{HashMap, HashSet};

use crate::base::database::{ColumnRef, TableRef};
use crate::sql::parse::{ConversionError, ConversionResult};

use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, Expression, OrderBy, ResultExpr, Slice,
};
use proofs_sql::Identifier;

#[derive(Default)]
pub struct QueryContext {
    slice: Option<Slice>,
    order_by: Vec<OrderBy>,
    table: Option<TableRef>,
    agg_result_exprs: Vec<usize>,
    fixed_columns_counter: usize,
    non_agg_result_exprs: Vec<usize>,
    referenced_columns_counter: usize,
    group_by_exprs: HashSet<Identifier>,
    where_expr: Option<Box<Expression>>,
    result_schema: Vec<AliasedResultExpr>,
    result_column_references: HashSet<Identifier>,
    column_mapping: HashMap<Identifier, ColumnRef>,
}

impl QueryContext {
    pub fn set_table(&mut self, table: TableRef) {
        assert!(self.table.is_none());
        self.table = Some(table);
    }

    pub fn current_table(&self) -> &TableRef {
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
        self.referenced_columns_counter += 1;
        self.column_mapping.insert(column, column_ref);
    }

    pub fn push_schema_column(&mut self, column: AliasedResultExpr, is_agg: bool) {
        self.result_schema.push(column.clone());

        if is_agg {
            self.agg_result_exprs.push(self.result_schema.len() - 1);
        } else {
            self.non_agg_result_exprs.push(self.result_schema.len() - 1);
        }
    }

    pub fn push_result_column_reference(&mut self, column: Identifier) {
        self.result_column_references.insert(column);
    }

    pub fn push_group_by(&mut self, column: Identifier) {
        self.group_by_exprs.insert(column);
    }

    pub fn push_order_by(&mut self, order_by: OrderBy) {
        self.order_by.push(order_by);
    }

    pub fn fix_columns_counter(&mut self) {
        self.fixed_columns_counter = self.referenced_columns_counter;
    }

    pub fn validate_columns_counter(&mut self) -> ConversionResult<()> {
        if self.referenced_columns_counter == self.fixed_columns_counter {
            return Err(ConversionError::InvalidExpression(
                "At least one column must be referenced".to_string(),
            ));
        }
        self.fix_columns_counter();
        Ok(())
    }

    pub fn get_result_schema(&self) -> ConversionResult<Vec<AliasedResultExpr>> {
        assert!(
            !self.result_schema.is_empty(),
            "Result schema must not be empty"
        );

        // We need to check that each column alias is unique
        for col in &self.result_schema {
            if self
                .result_schema
                .iter()
                .map(|c| (c.alias == col.alias) as u64)
                .sum::<u64>()
                != 1
            {
                return Err(ConversionError::DuplicateColumnAlias(col.alias.to_string()));
            }
        }

        Ok(self.result_schema.clone())
    }

    pub fn get_order_by(&self) -> ConversionResult<Vec<OrderBy>> {
        // order by must reference only aliases in the result schema
        for by_expr in &self.order_by {
            self.result_schema
                .iter()
                .find(|col| col.alias == by_expr.expr)
                .ok_or(ConversionError::InvalidOrderByError(
                    by_expr.expr.as_str().to_string(),
                ))?;
        }

        Ok(self.order_by.clone())
    }

    pub fn get_slice(&self) -> &Option<Slice> {
        &self.slice
    }

    pub fn get_agg_result_exprs(&self) -> ConversionResult<Vec<AliasedResultExpr>> {
        if !self.agg_result_exprs.is_empty() {
            // We can't aggregate without specifying a group by column yet
            if self.group_by_exprs.is_empty() {
                return Err(ConversionError::MissingGroupByError);
            }
        }

        // We need to remap count(*) to count(group_by_column)
        // since polars has issues with duplicated aliases when using count(*)
        Ok(self
            .agg_result_exprs
            .iter()
            .map(|agg_idx| {
                let agg = &self.result_schema[*agg_idx];
                match agg.expr {
                    ResultExpr::Agg(AggExpr::CountALL) => AliasedResultExpr {
                        expr: ResultExpr::Agg(AggExpr::Count(Box::new(Expression::Column(
                            *self.group_by_exprs.iter().next().unwrap(),
                        )))),
                        alias: agg.alias,
                    },
                    _ => agg.clone(),
                }
            })
            .collect())
    }

    pub fn get_group_by(&self) -> ConversionResult<Vec<(Identifier, Option<Identifier>)>> {
        if self.group_by_exprs.is_empty() {
            return Ok(Vec::new());
        }

        let mut transform_group_by_exprs = Vec::new();

        // We need to add the group by columns that are part of the result schema.
        for col_idx in self.non_agg_result_exprs.iter() {
            let col = &self.result_schema[*col_idx];
            let col_id: Identifier = *col
                .try_as_identifier()
                .ok_or(ConversionError::InvalidGroupByResultColumnError)?;

            // We need to check that each non aggregated result column
            // is referenced in the group by clause.
            if self
                .group_by_exprs
                .iter()
                .any(|group_by| *group_by == col_id)
            {
                // note: `col.alias` here implies that this group by will appear
                // in the result schema using `col.alias`.
                transform_group_by_exprs.push((col_id, Some(col.alias)));
            } else {
                return Err(ConversionError::InvalidGroupByResultColumnError);
            }
        }

        // We need to add the group by columns that are not part of the result schema.
        for group_by in self.group_by_exprs.iter() {
            if !self
                .non_agg_result_exprs
                .iter()
                .any(|col_idx| self.result_schema[*col_idx].try_as_identifier() == Some(group_by))
            {
                // note: `None` here implies that the this group by will be
                // filtered out of the result schema.
                transform_group_by_exprs.push((*group_by, None))
            }
        }

        Ok(transform_group_by_exprs)
    }

    pub fn get_referenced_columns(&self) -> HashSet<Identifier> {
        self.result_column_references.clone()
    }

    pub fn get_column_mapping(&self) -> HashMap<Identifier, ColumnRef> {
        self.column_mapping.clone()
    }
}
