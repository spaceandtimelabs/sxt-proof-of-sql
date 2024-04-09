use super::QueryContext;
use crate::{
    base::{
        database::{ColumnRef, ColumnType, SchemaAccessor, TableRef},
        math::decimal::Precision,
    },
    sql::parse::{ConversionError, ConversionResult},
};
use proofs_sql::{
    intermediate_ast::{
        AggregationOperator, AliasedResultExpr, BinaryOperator, Expression, Literal, OrderBy,
        SelectResultExpr, Slice, TableExpression, UnaryOperator,
    },
    Identifier, ResourceId,
};
use std::ops::Deref;

pub struct QueryContextBuilder<'a> {
    context: QueryContext,
    schema_accessor: &'a dyn SchemaAccessor,
}

// Public interface
impl<'a> QueryContextBuilder<'a> {
    pub fn new(schema_accessor: &'a dyn SchemaAccessor) -> Self {
        Self {
            context: QueryContext::default(),
            schema_accessor,
        }
    }

    pub fn visit_table_expr(
        mut self,
        table_expr: Vec<Box<TableExpression>>,
        default_schema: Identifier,
    ) -> Self {
        assert_eq!(table_expr.len(), 1);
        match *table_expr[0] {
            TableExpression::Named { table, schema } => {
                self.context.set_table_ref(TableRef::new(ResourceId::new(
                    schema.unwrap_or(default_schema),
                    table,
                )));
            }
        }
        self
    }

    pub fn visit_where_expr(
        mut self,
        mut where_expr: Option<Box<Expression>>,
    ) -> ConversionResult<Self> {
        if let Some(expr) = where_expr.as_deref_mut() {
            self.visit_expr(expr)?;
        }
        self.context.set_where_expr(where_expr);
        Ok(self)
    }

    pub fn visit_result_exprs(
        mut self,
        result_exprs: Vec<SelectResultExpr>,
    ) -> ConversionResult<Self> {
        self.context.toggle_result_scope();
        for column in result_exprs {
            match column {
                SelectResultExpr::ALL => self.visit_select_all_expr()?,
                SelectResultExpr::AliasedResultExpr(expr) => self.visit_aliased_expr(expr)?,
            }
        }
        self.context.toggle_result_scope();

        Ok(self)
    }

    pub fn visit_order_by_exprs(mut self, order_by_exprs: Vec<OrderBy>) -> Self {
        self.context.set_order_by_exprs(order_by_exprs);
        self
    }

    pub fn visit_slice_expr(mut self, slice: Option<Slice>) -> Self {
        self.context.set_slice_expr(slice);
        self
    }

    pub fn visit_group_by_exprs(
        mut self,
        group_by_exprs: Vec<Identifier>,
    ) -> ConversionResult<Self> {
        for id in &group_by_exprs {
            self.visit_column_identifier(*id)?;
        }
        self.context.set_group_by_exprs(group_by_exprs);
        Ok(self)
    }

    pub fn build(self) -> ConversionResult<QueryContext> {
        Ok(self.context)
    }
}

// Private interface
impl<'a> QueryContextBuilder<'a> {
    fn lookup_schema(&self) -> Vec<(Identifier, ColumnType)> {
        let table_ref = self.context.get_table_ref();
        let columns = self.schema_accessor.lookup_schema(*table_ref);
        assert!(!columns.is_empty(), "At least one column must exist");
        columns
    }

    fn visit_select_all_expr(&mut self) -> ConversionResult<()> {
        for (column_name, _) in self.lookup_schema() {
            let col_expr = Expression::Column(column_name);
            self.visit_aliased_expr(AliasedResultExpr::new(col_expr, column_name))?;
        }
        Ok(())
    }

    fn visit_aliased_expr(&mut self, mut aliased_expr: AliasedResultExpr) -> ConversionResult<()> {
        self.visit_expr(aliased_expr.expr.as_mut())?;
        self.context.push_aliased_result_expr(aliased_expr)?;
        Ok(())
    }

    /// Visits the expression and returns its data type, or None if it's a boolean expression.
    ///
    /// This function accepts the expression as a mutable reference because certain expressions
    /// require replacement, such as `count(*)` being replaced with `count(some_column)`.
    fn visit_expr(&mut self, expr: &mut Expression) -> ConversionResult<Option<ColumnType>> {
        match expr {
            Expression::Wildcard => self.visit_wildcard_expr(expr),
            Expression::Literal(literal) => Ok(Some(self.visit_literal(literal.deref())?)),
            Expression::Column(_) => self.visit_column_expr(expr),
            Expression::Unary { op, expr } => self.visit_unary_expr(op, expr),
            Expression::Binary { op, left, right } => self.visit_binary_expr(op, left, right),
            Expression::Aggregation { op, expr } => self.visit_agg_expr(op, expr),
        }
    }

    fn visit_wildcard_expr(
        &mut self,
        expr: &mut Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        let (col_name, col_type) = match self.context.get_any_result_column_ref() {
            Some((name, col_type)) => (name, col_type),
            None => self.lookup_schema().into_iter().next().unwrap(),
        };

        // Replace `count(*)` with `count(col_name)` to overcome limitations in Polars.
        *expr = Expression::Column(col_name);

        // Visit the column to ensure its inclusion in the result column set.
        self.visit_column_expr(expr)?;

        // Return the column type
        Ok(Some(col_type))
    }

    fn visit_column_expr(&mut self, expr: &mut Expression) -> ConversionResult<Option<ColumnType>> {
        let identifier = match expr {
            Expression::Column(identifier) => *identifier,
            _ => panic!("Must be a column expression"),
        };

        // When using `group by` clauses, result columns outside aggregation
        // need to be remapped to an aggregation function. This prevents Polars
        // from returning lists when the expected result is single elements.
        if self.context.is_in_group_by_exprs(&identifier)? {
            *expr = *Expression::Column(identifier).first();
        }

        Ok(Some(self.visit_column_identifier(identifier)?))
    }

    fn visit_binary_expr(
        &mut self,
        op: &BinaryOperator,
        left: &mut Expression,
        right: &mut Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        match op {
            BinaryOperator::And | BinaryOperator::Or => {
                let left_dtype = self.visit_expr(left)?;
                let right_dtype = self.visit_expr(right)?;
                assert!(left_dtype.is_none() && right_dtype.is_none());
                Ok(None)
            }
            BinaryOperator::Equal => {
                self.visit_equal_expr(left, right)?;
                Ok(None)
            }
            BinaryOperator::GreaterThanOrEqual | BinaryOperator::LessThanOrEqual => {
                self.visit_inequality_expr(left, right)?;
                Ok(None)
            }
            BinaryOperator::Multiply
            | BinaryOperator::Division
            | BinaryOperator::Subtract
            | BinaryOperator::Add => Ok(Some(self.visit_arithmetic_expr(left, right)?)),
        }
    }

    fn visit_arithmetic_expr(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
    ) -> ConversionResult<ColumnType> {
        let err_msg = "parser must prevent arithmetic operations with bool expresions";
        let left_dtype = self.visit_expr(left)?.expect(err_msg);
        let right_dtype = self.visit_expr(right)?.expect(err_msg);

        check_dtypes(left_dtype, right_dtype)?;

        if ColumnType::VarChar == left_dtype {
            return Err(ConversionError::InvalidExpression(format!(
                "arithmetic operations with data type {} is not supported",
                left_dtype
            )));
        }

        Ok(left_dtype)
    }

    fn visit_unary_expr(
        &mut self,
        op: &UnaryOperator,
        expr: &mut Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        match op {
            UnaryOperator::Not => {
                let dtype = self.visit_expr(expr)?;
                assert!(
                    dtype.is_none(),
                    "Unary not must be applied to a bool expression for now"
                );
                Ok(None)
            }
        }
    }

    fn visit_agg_expr(
        &mut self,
        op: &AggregationOperator,
        expr: &mut Expression,
    ) -> ConversionResult<Option<ColumnType>> {
        self.context.set_in_agg_scope(true)?;

        let expr_dtype = self.visit_expr(expr)?;

        // We only support sum/max/min aggregations on numeric columns.
        if op != &AggregationOperator::Count && expr_dtype == Some(ColumnType::VarChar) {
            let expr_dtype = expr_dtype.unwrap();
            return Err(ConversionError::non_numeric_expr_in_agg(
                expr_dtype.to_string(),
                op.to_string(),
            ));
        }

        self.context.set_in_agg_scope(false)?;

        // Count aggregation always results in an integer type
        if op == &AggregationOperator::Count {
            Ok(Some(ColumnType::BigInt))
        } else {
            Ok(expr_dtype)
        }
    }

    fn visit_equal_expr(&mut self, left: &Expression, right: &Expression) -> ConversionResult<()> {
        let left_dtype = match left {
            Expression::Column(identifier) => self.visit_column_identifier(*identifier)?,
            _ => panic!("Left side of comparison expression must be a column"),
        };
        let right_dtype = match right {
            Expression::Literal(literal) => self.visit_literal(literal)?,
            _ => panic!("Right side of comparison expression must be a literal"),
        };
        check_dtypes(left_dtype, right_dtype)?;
        Ok(())
    }

    fn visit_inequality_expr(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> ConversionResult<()> {
        let left_dtype = match left {
            Expression::Column(identifier) => self.visit_column_identifier(*identifier)?,
            _ => panic!("Left side of comparison expression must be a column"),
        };
        let right_dtype = match right {
            Expression::Literal(literal) => self.visit_literal(literal)?,
            _ => panic!("Right side of comparison expression must be a literal"),
        };
        check_dtypes(left_dtype, right_dtype)?;

        // Currently, we only support comparisons between bigint/int64 cols and int128 literals
        if ColumnType::BigInt != left_dtype || ColumnType::Int128 != right_dtype {
            return Err(ConversionError::InvalidExpression(format!(
                "Currently >= and <= operators only support BigInt. Unsupported comparison between '{}' and '{}'", left_dtype, right_dtype
            )));
        }
        Ok(())
    }

    fn visit_literal(&self, literal: &Literal) -> Result<ColumnType, ConversionError> {
        match literal {
            Literal::Int128(_) => Ok(ColumnType::Int128),
            Literal::VarChar(_) => Ok(ColumnType::VarChar),
            Literal::Decimal(d) => {
                let precision =
                    Precision::new(d.precision()).map_err(ConversionError::PrecisionParseError)?;
                Ok(ColumnType::Decimal75(precision, d.scale()))
            }
        }
    }

    fn visit_column_identifier(&mut self, column_name: Identifier) -> ConversionResult<ColumnType> {
        let table_ref = self.context.get_table_ref();
        let column_type = self.schema_accessor.lookup_column(*table_ref, column_name);

        let column_type = column_type.ok_or_else(|| {
            ConversionError::MissingColumn(Box::new(column_name), Box::new(table_ref.resource_id()))
        })?;

        let column = ColumnRef::new(*table_ref, column_name, column_type);

        self.context.push_column_ref(column_name, column);

        Ok(column_type)
    }
}

fn are_types_compatible(left_dtype: &ColumnType, right_dtype: &ColumnType) -> bool {
    matches!(
        (left_dtype, right_dtype),
        (ColumnType::BigInt, ColumnType::BigInt)
            | (ColumnType::BigInt, ColumnType::Int128)
            | (
                ColumnType::BigInt | ColumnType::Int128,
                ColumnType::Decimal75(_, _)
            )
            | (ColumnType::Int128, ColumnType::Int128)
            | (ColumnType::Int128, ColumnType::BigInt)
            | (ColumnType::Decimal75(_, _), ColumnType::Decimal75(_, _))
            | (
                ColumnType::Decimal75(_, _),
                ColumnType::BigInt | ColumnType::Int128
            )
            | (ColumnType::VarChar, ColumnType::VarChar)
    )
}

fn check_dtypes(left_dtype: ColumnType, right_dtype: ColumnType) -> ConversionResult<()> {
    if are_types_compatible(&left_dtype, &right_dtype) {
        Ok(())
    } else {
        Err(ConversionError::DataTypeMismatch(
            left_dtype.to_string(),
            right_dtype.to_string(),
        ))
    }
}
