use super::{ConversionError, ConversionResult, QueryContext};
use crate::base::{
    database::{
        try_add_subtract_column_types, try_multiply_column_types, ColumnNullability, ColumnRef,
        ColumnType, SchemaAccessor, TableRef,
    },
    math::decimal::Precision,
};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use proof_of_sql_parser::{
    intermediate_ast::{
        AggregationOperator, AliasedResultExpr, BinaryOperator, Expression, Literal, OrderBy,
        SelectResultExpr, Slice, TableExpression, UnaryOperator,
    },
    Identifier, ResourceId,
};

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

    #[allow(clippy::vec_box, clippy::missing_panics_doc)]
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
    #[allow(
        clippy::missing_panics_doc,
        reason = "The assertion ensures there is at least one column, and this is a fundamental requirement for schema retrieval."
    )]
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

    fn visit_aliased_expr(&mut self, aliased_expr: AliasedResultExpr) -> ConversionResult<()> {
        self.visit_expr(&aliased_expr.expr)?;
        self.context.push_aliased_result_expr(aliased_expr)?;
        Ok(())
    }

    /// Visits the expression and returns its data type.
    fn visit_expr(&mut self, expr: &Expression) -> ConversionResult<ColumnType> {
        match expr {
            Expression::Wildcard => Ok(ColumnType::BigInt(ColumnNullability::NotNullable)), // Since COUNT(*) = COUNT(1)
            Expression::Literal(literal) => self.visit_literal(literal),
            Expression::Column(_) => self.visit_column_expr(expr),
            Expression::Unary { op, expr } => self.visit_unary_expr(*op, expr),
            Expression::Binary { op, left, right } => self.visit_binary_expr(*op, left, right),
            Expression::Aggregation { op, expr } => self.visit_agg_expr(*op, expr),
        }
    }

    /// # Panics
    /// Panics if the expression is not a column expression.
    fn visit_column_expr(&mut self, expr: &Expression) -> ConversionResult<ColumnType> {
        let identifier = match expr {
            Expression::Column(identifier) => *identifier,
            _ => panic!("Must be a column expression"),
        };

        self.visit_column_identifier(identifier)
    }

    fn visit_binary_expr(
        &mut self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> ConversionResult<ColumnType> {
        let left_dtype = self.visit_expr(left)?;
        let right_dtype = self.visit_expr(right)?;
        check_dtypes(left_dtype, right_dtype, op)?;
        match op {
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Equal
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::LessThanOrEqual => {
                Ok(ColumnType::Boolean(ColumnNullability::NotNullable))
            }
            BinaryOperator::Multiply
            | BinaryOperator::Division
            | BinaryOperator::Subtract
            | BinaryOperator::Add => Ok(left_dtype),
        }
    }

    fn visit_unary_expr(
        &mut self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> ConversionResult<ColumnType> {
        match op {
            UnaryOperator::Not => {
                let dtype = self.visit_expr(expr)?;
                if let ColumnType::Boolean(_) = dtype {
                    Ok(dtype)
                } else {
                    Err(ConversionError::InvalidDataType {
                        expected: ColumnType::Boolean(ColumnNullability::NotNullable),
                        actual: dtype,
                    })
                }
            }
        }
    }

    fn visit_agg_expr(
        &mut self,
        op: AggregationOperator,
        expr: &Expression,
    ) -> ConversionResult<ColumnType> {
        self.context.set_in_agg_scope(true)?;
        let expr_dtype = self.visit_expr(expr)?;

        match (op, expr_dtype) {
            (AggregationOperator::Count, _) => {
                self.context.set_in_agg_scope(false)?;
                Ok(ColumnType::BigInt(ColumnNullability::NotNullable))
            }
            (_, ColumnType::VarChar(_)) => Err(ConversionError::non_numeric_expr_in_agg(
                expr_dtype.to_string(),
                op.to_string(),
            )),
            (_, _) => {
                self.context.set_in_agg_scope(false)?;
                Ok(expr_dtype)
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn visit_literal(&self, literal: &Literal) -> Result<ColumnType, ConversionError> {
        let meta = ColumnNullability::NotNullable;
        match literal {
            Literal::Boolean(_) => Ok(ColumnType::Boolean(meta)),
            Literal::BigInt(_) => Ok(ColumnType::BigInt(meta)),
            Literal::Int128(_) => Ok(ColumnType::Int128(meta)),
            Literal::VarChar(_) => Ok(ColumnType::VarChar(meta)),
            Literal::Decimal(d) => {
                let precision = Precision::new(d.precision())?;
                Ok(ColumnType::Decimal75(meta, precision, d.scale()))
            }
            Literal::Timestamp(its) => Ok(ColumnType::TimestampTZ(
                meta,
                its.timeunit(),
                its.timezone(),
            )),
        }
    }

    fn visit_column_identifier(&mut self, column_name: Identifier) -> ConversionResult<ColumnType> {
        let table_ref = self.context.get_table_ref();
        let column_type = self.schema_accessor.lookup_column(*table_ref, column_name);

        let column_type = column_type.ok_or_else(|| ConversionError::MissingColumn {
            identifier: Box::new(column_name),
            resource_id: Box::new(table_ref.resource_id()),
        })?;

        let column = ColumnRef::new(*table_ref, column_name, column_type);

        self.context.push_column_ref(column_name, column);

        Ok(column_type)
    }
}

/// TODO: add docs
pub(crate) fn type_check_binary_operation(
    left_dtype: &ColumnType,
    right_dtype: &ColumnType,
    binary_operator: BinaryOperator,
) -> bool {
    match binary_operator {
        BinaryOperator::And | BinaryOperator::Or => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::Boolean(_), ColumnType::Boolean(_))
            )
        }
        BinaryOperator::Equal => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::VarChar(_), ColumnType::VarChar(_))
                    | (
                        ColumnType::TimestampTZ(_, _, _),
                        ColumnType::TimestampTZ(_, _, _)
                    )
                    | (ColumnType::Boolean(_), ColumnType::Boolean(_))
                    | (_, ColumnType::Scalar(_))
                    | (ColumnType::Scalar(_), _)
            ) || (left_dtype.is_numeric() && right_dtype.is_numeric())
        }
        BinaryOperator::GreaterThanOrEqual | BinaryOperator::LessThanOrEqual => {
            match (left_dtype, right_dtype) {
                (ColumnType::VarChar(_), _) | (_, ColumnType::VarChar(_)) => return false,
                _ => {}
            };
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(_, precision, _) = left_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            if let ColumnType::Decimal75(_, precision, _) = right_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean(_), ColumnType::Boolean(_))
                        | (
                            ColumnType::TimestampTZ(_, _, _),
                            ColumnType::TimestampTZ(_, _, _)
                        )
                )
        }
        BinaryOperator::Add => {
            try_add_subtract_column_types(*left_dtype, *right_dtype, BinaryOperator::Add).is_ok()
        }
        BinaryOperator::Subtract => {
            try_add_subtract_column_types(*left_dtype, *right_dtype, BinaryOperator::Subtract)
                .is_ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(*left_dtype, *right_dtype).is_ok(),
        BinaryOperator::Division => left_dtype.is_numeric() && right_dtype.is_numeric(),
    }
}

fn check_dtypes(
    left_dtype: ColumnType,
    right_dtype: ColumnType,
    binary_operator: BinaryOperator,
) -> ConversionResult<()> {
    if type_check_binary_operation(&left_dtype, &right_dtype, binary_operator) {
        Ok(())
    } else {
        Err(ConversionError::DataTypeMismatch {
            left_type: left_dtype.to_string(),
            right_type: right_dtype.to_string(),
        })
    }
}
