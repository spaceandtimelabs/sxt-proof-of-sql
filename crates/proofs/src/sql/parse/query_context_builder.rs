use std::ops::Deref;

use super::QueryContext;
use crate::base::database::SchemaAccessor;
use crate::base::database::TableRef;
use crate::base::database::{ColumnRef, ColumnType};
use crate::sql::parse::{ConversionError, ConversionResult};

use proofs_sql::intermediate_ast::BinaryOperator;
use proofs_sql::intermediate_ast::TableExpression;
use proofs_sql::intermediate_ast::UnaryOperator;
use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, Expression, Literal, OrderBy, ResultExpr, SelectResultExpr, Slice,
};
use proofs_sql::{Identifier, ResourceId};

pub struct QueryContextBuilder<'a> {
    context: QueryContext,
    referencing_results: bool,
    schema_accessor: &'a dyn SchemaAccessor,
}

impl<'a> QueryContextBuilder<'a> {
    pub fn new(schema_accessor: &'a dyn SchemaAccessor) -> Self {
        Self {
            context: QueryContext::default(),
            referencing_results: false,
            schema_accessor,
        }
    }

    pub fn visit_table_expression(
        mut self,
        table_expr: Vec<Box<TableExpression>>,
        default_schema: Identifier,
    ) -> Self {
        assert_eq!(table_expr.len(), 1);
        match *table_expr[0] {
            TableExpression::Named { table, schema } => {
                self.context.set_table(TableRef::new(ResourceId::new(
                    schema.unwrap_or(default_schema),
                    table,
                )));
            }
        }
        self
    }

    pub fn visit_where_expr(
        mut self,
        where_expr: Option<Box<Expression>>,
    ) -> ConversionResult<Self> {
        if let Some(expr) = &where_expr {
            self.visit_expression(expr.deref())?;
        }
        self.context.set_where_expr(where_expr);
        Ok(self)
    }

    /// Visit the expression and return the data type of the expression
    /// Returns None if the expression is a boolean expression
    fn visit_expression(
        &mut self,
        expression: &Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        match expression {
            Expression::Literal(literal) => Ok(Some(self.visit_literal(literal.deref()))),
            Expression::Column(identifier) => Ok(Some(self.visit_column_identifier(*identifier)?)),
            Expression::Unary { op, expr } => self.visit_unary_expression(op, expr),
            Expression::Binary { op, left, right } => {
                self.visit_binary_expression(op, left.deref(), right.deref())
            }
        }
    }

    fn visit_binary_expression(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        match op {
            BinaryOperator::And | BinaryOperator::Or => {
                let left_dtype = self.visit_expression(left)?;
                let right_dtype = self.visit_expression(right)?;
                assert!(left_dtype.is_none() && right_dtype.is_none());
                Ok(None)
            }
            BinaryOperator::Equal => {
                self.visit_equal_expression(left, right)?;
                Ok(None)
            }
            BinaryOperator::Multiply | BinaryOperator::Subtract | BinaryOperator::Add => {
                let left_dtype = self.visit_expression(left)?;
                let right_dtype = self.visit_expression(right)?;
                check_dtypes(
                    left_dtype.expect("Must not be a boolean expression"),
                    right_dtype.expect("Must not be a boolean expression"),
                )?;
                Ok(left_dtype)
            }
        }
    }

    fn visit_unary_expression(
        &mut self,
        op: &UnaryOperator,
        expr: &Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        match op {
            UnaryOperator::Not => {
                let dtype = self.visit_expression(expr)?;
                assert!(
                    dtype.is_none(),
                    "Unary not must be applied to a bool expression for now"
                );
                Ok(None)
            }
        }
    }

    fn visit_equal_expression(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> ConversionResult<()> {
        let left_dtype = match left {
            Expression::Column(identifier) => self.visit_column_identifier(*identifier)?,
            _ => panic!("Left side of comparison expression must be a column"),
        };
        let right_dtype = match right {
            Expression::Literal(literal) => self.visit_literal(literal.deref()),
            _ => panic!("Right side of comparison expression must be a literal"),
        };
        check_dtypes(left_dtype, right_dtype)?;
        Ok(())
    }

    fn visit_literal(&self, literal: &Literal) -> ColumnType {
        match literal {
            Literal::Int128(_) => ColumnType::Int128,
            Literal::VarChar(_) => ColumnType::VarChar,
        }
    }

    fn visit_column_identifier(&mut self, column_name: Identifier) -> ConversionResult<ColumnType> {
        let current_table = self.context.current_table();
        let column_type = self
            .schema_accessor
            .lookup_column(*current_table, column_name);

        let column_type = column_type.ok_or_else(|| {
            ConversionError::MissingColumnError(
                Box::new(column_name),
                Box::new(current_table.resource_id()),
            )
        })?;

        let column = ColumnRef::new(*current_table, column_name, column_type);

        // We need to keep track of those columns to build the filter result expression
        if self.referencing_results {
            self.context.push_result_column_reference(column_name);
        }

        self.context.push_column_ref(column_name, column);

        Ok(column_type)
    }

    pub fn visit_order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        for by in order_by {
            self.context.push_order_by(by);
        }
        self
    }

    pub fn visit_slice(mut self, slice: Option<Slice>) -> Self {
        self.context.set_slice(slice);
        self
    }

    pub fn visit_group_by(mut self, group_by: Vec<Identifier>) -> ConversionResult<Self> {
        self.referencing_results = true;
        for identifier in group_by {
            self.visit_column_identifier(identifier)?;
            self.context.push_group_by(identifier);
        }
        self.referencing_results = false;

        Ok(self)
    }

    fn visit_agg_expr(&mut self, agg_expr: AggExpr) -> ConversionResult<AggExpr> {
        match &agg_expr {
            AggExpr::Count(expr) => {
                self.context.fix_columns_counter();
                self.visit_expression(expr)?;
                self.context.validate_columns_counter()?;
            }
            AggExpr::Sum(expr) | AggExpr::Min(expr) | AggExpr::Max(expr) => {
                self.context.fix_columns_counter();
                let expr_dtype = self
                    .visit_expression(expr)?
                    .expect("Expression cannot be bool yet");

                // We only support sum/max/min aggregation on numeric columns
                if expr_dtype == ColumnType::VarChar {
                    return Err(ConversionError::NonNumericColumnAggregation("max/min/sum"));
                }
                self.context.validate_columns_counter()?;
            }
            AggExpr::CountALL => {}
        }
        Ok(agg_expr)
    }

    pub fn visit_result_columns(
        mut self,
        result_columns: Vec<SelectResultExpr>,
    ) -> ConversionResult<Self> {
        self.referencing_results = true;

        for column in result_columns {
            match column {
                SelectResultExpr::ALL => {
                    let current_table = self.context.current_table();
                    let columns = self.schema_accessor.lookup_schema(*current_table);
                    for (column_name, _) in columns {
                        self.visit_column_identifier(column_name)?;
                        self.context.push_schema_column(
                            AliasedResultExpr::from_non_agg_expr(
                                Expression::Column(column_name),
                                column_name,
                            ),
                            false,
                        );
                    }
                }
                SelectResultExpr::AliasedResultExpr(aliased_expr) => {
                    let is_agg = match &aliased_expr.expr {
                        ResultExpr::Agg(agg_expr) => {
                            self.visit_agg_expr(agg_expr.clone())?;
                            true
                        }
                        ResultExpr::NonAgg(expr) => {
                            self.context.fix_columns_counter();
                            self.visit_expression(expr)?;
                            self.context.validate_columns_counter()?;
                            false
                        }
                    };
                    self.context.push_schema_column(aliased_expr, is_agg);
                }
            }
        }

        self.referencing_results = false;

        Ok(self)
    }

    pub fn build(self) -> ConversionResult<QueryContext> {
        Ok(self.context)
    }
}

pub fn check_dtypes(left_dtype: ColumnType, right_dtype: ColumnType) -> ConversionResult<()> {
    match right_dtype {
        ColumnType::Int128 | ColumnType::BigInt => {
            // Integer literal is compatible with any integeger column type other than VarChar
            if left_dtype == ColumnType::VarChar {
                return Err(ConversionError::MismatchTypeError(
                    left_dtype.to_string(),
                    right_dtype.to_string(),
                ));
            }
        }
        ColumnType::VarChar => {
            // Varchar literal is only compatible wtih VarChar column type
            if left_dtype != ColumnType::VarChar {
                return Err(ConversionError::MismatchTypeError(
                    left_dtype.to_string(),
                    right_dtype.to_string(),
                ));
            }
        }
    }

    Ok(())
}
