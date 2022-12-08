use crate::base::database::SchemaAccessor;
use crate::base::scalar::IntoScalar;
use crate::sql::ast::{
    AndExpr, BoolExpr, ColumnRef, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::sql::parse::{ParseError, ParseResult};
use curve25519_dalek::scalar::Scalar;
use proofs_sql::intermediate_ast::{
    Expression, ResultColumn, SelectStatement, SetExpression, TableExpression,
};
use proofs_sql::symbols::Name;
use std::ops::Deref;

#[derive(Default)]
pub struct Converter {
    /// The current table in context
    current_table: Option<String>,
}

impl Converter {
    /// Convert an Intermediate AST into a Provable AST
    pub fn visit_intermediate_ast(
        &mut self,
        ast: &SelectStatement,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<FilterExpr> {
        self.visit_set_expression(ast.expr.deref(), schema_accessor)
    }
}

/// Visit intermediate ast
impl Converter {
    /// Convert a `SetExpression` into a `FilterExpr`
    fn visit_set_expression(
        &mut self,
        expr: &SetExpression,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<FilterExpr> {
        match expr {
            SetExpression::Query {
                columns,
                from,
                where_expr,
            } => {
                // we should always visit table_expr first, as we need to know the current table name
                let table = self.visit_table_expressions(&from[..]);
                let filter_result_expr_list =
                    self.visit_result_columns(&columns[..], schema_accessor)?;
                let where_clause =
                    self.visit_bool_expression(where_expr.deref(), schema_accessor)?;

                Ok(FilterExpr::new(
                    filter_result_expr_list,
                    table,
                    where_clause,
                ))
            }
        }
    }
}

/// Table expression
impl Converter {
    /// Convert a `TableExpression` into a TableExpr
    fn visit_table_expression(&mut self, table_expr: &TableExpression) -> TableExpr {
        match table_expr {
            TableExpression::Named { table, namespace } => {
                assert!(namespace.is_none());

                let name = table.as_str().to_string();

                self.current_table = Some(name.clone());

                TableExpr { name }
            }
        }
    }

    /// Convert a `TableExpression slice` into a `TableExpr`
    fn visit_table_expressions(&mut self, table_exprs: &[Box<TableExpression>]) -> TableExpr {
        assert!(table_exprs.len() == 1);

        self.visit_table_expression(table_exprs[0].deref())
    }
}

/// Result expression
impl Converter {
    /// Convert a `ResultColumn` into a `FilterResultExpr`
    fn visit_result_column(
        &self,
        result_column: &ResultColumn,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<FilterResultExpr> {
        match result_column {
            ResultColumn::Expr { expr, output_name } => {
                let result_expr = self.visit_column_identifier(expr, schema_accessor)?;
                let output_name = output_name.as_ref().map(|output| output.as_str());
                let output_name = output_name.unwrap_or(&result_expr.column_name).to_string();

                Ok(FilterResultExpr::new(result_expr, output_name))
            }
        }
    }

    /// Convert a `ResultColumn slice` into a `Vec<FilterResultExpr>`
    fn visit_result_columns(
        &self,
        result_columns: &[Box<ResultColumn>],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<Vec<FilterResultExpr>> {
        assert!(!result_columns.is_empty());

        let results: Result<Vec<_>, _> = result_columns
            .iter()
            .map(|result_column| self.visit_result_column(result_column.deref(), schema_accessor))
            .into_iter()
            .collect();

        results
    }
}

/// Where expression
impl Converter {
    /// Convert an `Expression` into a BoolExpr
    fn visit_bool_expression(
        &self,
        expression: &Expression,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<Box<dyn BoolExpr>> {
        match expression {
            Expression::Not { expr } => Ok(Box::new(NotExpr::new(
                self.visit_bool_expression(expr.deref(), schema_accessor)?,
            ))),

            Expression::And { left, right } => Ok(Box::new(AndExpr::new(
                self.visit_bool_expression(left.deref(), schema_accessor)?,
                self.visit_bool_expression(right.deref(), schema_accessor)?,
            ))),

            Expression::Or { left, right } => Ok(Box::new(OrExpr::new(
                self.visit_bool_expression(left.deref(), schema_accessor)?,
                self.visit_bool_expression(right.deref(), schema_accessor)?,
            ))),

            // TODO: check if the column and the literal have the same type.
            //       For instance, in the query `select A from T where B = 123`
            //       we should verify if both B and 123 have the same type
            //       (in the future, B could be varchar, or boolean, or any other type other than Int64).
            Expression::Equal { left, right } => Ok(Box::new(EqualsExpr::new(
                self.visit_column_identifier(left, schema_accessor)?,
                self.visit_literal(*right),
            ))),

            Expression::NotEqual { left, right } => {
                Ok(Box::new(NotExpr::new(Box::new(EqualsExpr::new(
                    self.visit_column_identifier(left, schema_accessor)?,
                    self.visit_literal(*right),
                )))))
            }
        }
    }
}

/// Tokens (literals and id's)
impl Converter {
    /// Convert a `i64` into a `Scalar`
    fn visit_literal(&self, literal: i64) -> Scalar {
        literal.into_scalar()
    }

    /// Convert a `Name` into an identifier string (i.e. a string)
    fn visit_column_identifier(
        &self,
        id: &Name,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ParseResult<ColumnRef> {
        let column_name = id.as_str().to_string();

        let current_table = self.current_table.as_deref().unwrap();

        let column_type = schema_accessor.lookup_column(current_table, &column_name);

        if column_type.is_none() {
            return Err(ParseError::MissingColumnError(format!(
                "Column {:?} is not found in table {:?}",
                column_name, current_table
            )));
        }

        Ok(ColumnRef {
            column_name,
            table_name: current_table.to_string(),
            namespace: None,
            column_type: column_type.unwrap(),
        })
    }
}
