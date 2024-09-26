use super::{where_expr_builder::WhereExprBuilder, ConversionError, EnrichedExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, LiteralValue, TableRef},
        map::IndexMap,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, TableExpr},
        proof_plans::FilterExec,
    },
};
use itertools::Itertools;
use proof_of_sql_parser::{intermediate_ast::Expression, Identifier};

pub struct FilterExecBuilder<C: Commitment> {
    table_expr: Option<TableExpr>,
    where_expr: Option<DynProofExpr<C>>,
    filter_result_expr_list: Vec<AliasedDynProofExpr<C>>,
    column_mapping: IndexMap<Identifier, ColumnRef>,
}

// Public interface
impl<C: Commitment> FilterExecBuilder<C> {
    pub fn new(column_mapping: IndexMap<Identifier, ColumnRef>) -> Self {
        Self {
            table_expr: None,
            where_expr: None,
            filter_result_expr_list: vec![],
            column_mapping,
        }
    }

    pub fn add_table_expr(mut self, table_ref: TableRef) -> Self {
        self.table_expr = Some(TableExpr { table_ref });
        self
    }

    pub fn add_where_expr(
        mut self,
        where_expr: Option<Box<Expression>>,
    ) -> Result<Self, ConversionError> {
        self.where_expr = WhereExprBuilder::new(&self.column_mapping).build(where_expr)?;
        Ok(self)
    }

    pub fn add_result_columns(mut self, columns: &[EnrichedExpr<C>]) -> Self {
        // If a column is provable, add it to the filter result expression list
        // If at least one column is non-provable, add all columns from the column mapping to the filter result expression list
        let mut has_nonprovable_column = false;
        for enriched_expr in columns {
            if let Some(plan) = &enriched_expr.dyn_proof_expr {
                self.filter_result_expr_list.push(AliasedDynProofExpr {
                    expr: plan.clone(),
                    alias: enriched_expr.residue_expression.alias,
                });
            } else {
                has_nonprovable_column = true;
            }
        }
        if has_nonprovable_column {
            // Has to keep them sorted to have deterministic order for tests
            for alias in self.column_mapping.keys().sorted() {
                //TODO: add panic docs
                let column_ref = self.column_mapping.get(alias).unwrap();
                self.filter_result_expr_list.push(AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(*column_ref),
                    alias: *alias,
                });
            }
        }
        self
    }

    pub fn build(self) -> FilterExec<C> {
        FilterExec::new(
            self.filter_result_expr_list,
            self.table_expr.expect("Table expr is required"),
            self.where_expr
                .unwrap_or_else(|| DynProofExpr::new_literal(LiteralValue::Boolean(true))),
        )
    }
}
