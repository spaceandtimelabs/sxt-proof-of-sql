use super::{ConversionError, ConversionResult, QueryContext};
use crate::base::{database::{
    try_add_subtract_column_types, try_multiply_column_types, ColumnRef, ColumnType,
    SchemaAccessor, TableRef,
}, math::decimal::Precision, resource_id::ResourceId, utility};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use ark_std::iterable::Iterable;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{Table, BinaryOperator, UnaryOperator, Expr, Value, OrderBy, Ident, TableWithJoins, SelectItem, GroupByExpr, OrderByExpr};
use crate::sql::parse::query_context::Slice;

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
        table_expr: Vec<TableWithJoins>,
        default_schema: Ident,
    ) -> Self {
        assert_eq!(table_expr.len(), 1);
        match *table_expr {
            Table { ref table_name, ref schema_name } => {
                self.context.set_table_ref(TableRef::new(ResourceId::new(
                    schema_name.clone().map(utility::ident).unwrap_or(default_schema),
                    table_name.to_string(),
                )));
            }
        }
        self
    }

    pub fn visit_where_expr(
        mut self,
        mut where_expr: Option<Expr>,
    ) -> ConversionResult<Self> {
        if let Some(expr) = where_expr.as_deref_mut() {
            self.visit_expr(expr)?;
        }
        self.context.set_where_expr(where_expr);
        Ok(self)
    }

    pub fn visit_result_exprs(
        mut self,
        result_exprs: Vec<SelectItem>,
    ) -> ConversionResult<Self> {
        self.context.toggle_result_scope();
        for column in result_exprs {
            match column {
                SelectItem::Wildcard(_) => self.visit_select_all_expr()?,
                SelectItem::UnnamedExpr(ref expr @ Expr::Identifier(ident) ) => {
                    self.visit_aliased_expr(AliasedResultExpr {
                        alias: ident,
                        expr: Box::new(expr.clone()),
                    })?
                },
                SelectItem::ExprWithAlias(expr, alias) => self.visit_aliased_expr(AliasedResultExpr {
                    alias,
                    expr
                })?,
            }
        }
        self.context.toggle_result_scope();

        Ok(self)
    }

    pub fn visit_order_by(mut self, order_by: Vec<OrderBy>) -> Self {
        self.context.set_order_by_exprs(order_by_exprs);
        self
    }
    pub fn visit_order_by_exprs(mut self, order_by_exprs: Vec<OrderByExpr>) -> Self {
        self.context.set_order_by_exprs(order_by_exprs);
        self
    }

    pub fn visit_slice_expr(mut self, slice: Option<Slice>) -> Self {
        self.context.set_slice_expr(slice);
        self
    }

    pub fn visit_group_by_exprs(
        mut self,
        group_by_exprs: GroupByExpr,
    ) -> ConversionResult<Self> {

        let mut group_by_idents: Vec<Ident> = vec![];

        match group_by_exprs {
            GroupByExpr::Expressions( exprs, _) => {
                for expr in exprs {

                    if let Expr::Identifier(ident) = expr {

                        self.visit_column_identifier(&ident)?;
                        group_by_idents.push(ident);
                    }
                }
            },
            _ => panic!("Unsupported groupby: {group_by_exprs}")
        };
        self.context.set_group_by_exprs(group_by_idents);
        Ok(self)
    }

    pub fn build(self) -> ConversionResult<QueryContext> {
        Ok(self.context)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
/// An expression with an alias e.g. `a + 1 AS b`
pub struct AliasedResultExpr {
    /// The expression e.g. `a + 1`, `COUNT(*)`, etc.
    pub expr: Box<Expr>,
    /// The alias e.g. `count` in `COUNT(*) AS count`
    pub alias: Ident,
}


// Private interface
impl<'a> QueryContextBuilder<'a> {
    #[allow(
        clippy::missing_panics_doc,
        reason = "The assertion ensures there is at least one column, and this is a fundamental requirement for schema retrieval."
    )]
    fn lookup_schema(&self) -> Vec<(Ident, ColumnType)> {
        let table_ref = self.context.get_table_ref();
        let columns = self.schema_accessor.lookup_schema(table_ref.clone());
        assert!(!columns.is_empty(), "At least one column must exist");
        columns
    }

    fn visit_select_all_expr(&mut self) -> ConversionResult<()> {
        for (column_name, _) in self.lookup_schema() {
            let col_expr = Box::new(Expr::Identifier(column_name.clone()));
            self.visit_aliased_expr(AliasedResultExpr {expr: col_expr, alias: column_name})?;
        }
        Ok(())
    }

    fn visit_aliased_expr(&mut self, aliased_expr: AliasedResultExpr) -> ConversionResult<()> {
        self.visit_expr(&aliased_expr.expr)?;
        self.context.push_aliased_result_expr(aliased_expr)?;
        Ok(())
    }

    /// Visits the expression and returns its data type.
    fn visit_expr(&mut self, expr: &Expr) -> ConversionResult<ColumnType> {
        match expr {
            Expr::Wildcard => Ok(ColumnType::BigInt), // Since COUNT(*) = COUNT(1)
            Expr::Value(literal) => self.visit_literal(literal),
            Expr::Identifier(_)  => self.visit_column_expr(expr),
            Expr::UnaryOp { op, expr } => self.visit_unary_expr(*op, expr),
            Expr::BinaryOp { op, left, right } => self.visit_binary_expr(op.clone(), left, right),
            Expr::Aggregation { op, expr } => self.visit_agg_expr(*op, expr),
        }
    }

    /// # Panics
    /// Panics if the expression is not a column expression.
    fn visit_column_expr(&mut self, expr: &Expr) -> ConversionResult<ColumnType> {
        let identifier = match expr {
            Expr::Identifier(identifier) => identifier,
            _ => panic!("Must be a column expression"),
        };

        self.visit_column_identifier(identifier)
    }

    fn visit_binary_expr(
        &mut self,
        op: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> ConversionResult<ColumnType> {
        let left_dtype = self.visit_expr(left)?;
        let right_dtype = self.visit_expr(right)?;
        check_dtypes(left_dtype, right_dtype, op.clone())?;
        match op {
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Eq
            | BinaryOperator::GtEq
            | BinaryOperator::Gt
            | BinaryOperator::Lt
            | BinaryOperator::LtEq => Ok(ColumnType::Boolean),
            BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Minus
            | BinaryOperator::Plus => Ok(left_dtype),
            _ => panic!("Unmapped binary operator: {op}"),
        }
    }

    fn visit_unary_expr(
        &mut self,
        op: UnaryOperator,
        expr: &Expr,
    ) -> ConversionResult<ColumnType> {
        match op {
            UnaryOperator::Not => {
                let dtype = self.visit_expr(expr)?;
                if dtype != ColumnType::Boolean {
                    return Err(ConversionError::InvalidDataType {
                        expected: ColumnType::Boolean,
                        actual: dtype,
                    });
                }
                Ok(ColumnType::Boolean)
            },
            _ => panic!("Unmapped unary operator: {op}"),
        }
    }

    fn visit_agg_expr(
        &mut self,
        op: AggregationOperator,
        expr: &Expr,
    ) -> ConversionResult<ColumnType> {
        self.context.set_in_agg_scope(true)?;

        let expr_dtype = self.visit_expr(expr)?;

        // We only support sum/max/min aggregations on numeric columns.
        if op != AggregationOperator::Count && expr_dtype == ColumnType::VarChar {
            return Err(ConversionError::non_numeric_expr_in_agg(
                expr_dtype.to_string(),
                op.to_string(),
            ));
        }

        self.context.set_in_agg_scope(false)?;

        // Count aggregation always results in an integer type
        if op == AggregationOperator::Count {
            Ok(ColumnType::BigInt)
        } else {
            Ok(expr_dtype)
        }
    }

    #[allow(clippy::unused_self)]
    fn visit_literal(&self, literal: &Value) -> Result<ColumnType, ConversionError> {
        match literal {
            Value::Boolean(_) => Ok(ColumnType::Boolean),
            Value::Number(_, true) => Ok(ColumnType::BigInt),
            Value::Number(txt, false)  if   txt    => Ok(ColumnType::Int),
            Value::Int128(_) => Ok(ColumnType::Int128),
            Value::TripleDoubleQuotedString(_) => Ok(ColumnType::VarChar),
            Value::Decimal(d) => {
                let precision = Precision::new(d.precision())?;
                Ok(ColumnType::Decimal75(precision, d.scale()))
            }
            Value::Timestamp(its) => Ok(ColumnType::TimestampTZ(its.timeunit(), its.timezone())),
        }
    }

    fn visit_column_identifier(&mut self, column_name: &Ident) -> ConversionResult<ColumnType> {
        let table_ref = self.context.get_table_ref();
        let column_type = self.schema_accessor.lookup_column(table_ref.clone(), column_name);

        let column_type = column_type.ok_or_else(|| ConversionError::MissingColumn {
            identifier: Box::new(column_name.to_owned()),
            resource_id: Box::new(table_ref.resource_id()),
        })?;

        let column = ColumnRef::new(table_ref.clone(), column_name.clone(), column_type);

        self.context.push_column_ref(column_name.clone(), column);

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
                (ColumnType::Boolean, ColumnType::Boolean)
            )
        }
        BinaryOperator::Eq => {
            matches!(
                (left_dtype, right_dtype),
                (ColumnType::VarChar, ColumnType::VarChar)
                    | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                    | (ColumnType::Boolean, ColumnType::Boolean)
                    | (_, ColumnType::Scalar)
                    | (ColumnType::Scalar, _)
            ) || (left_dtype.is_numeric() && right_dtype.is_numeric())
        }
        BinaryOperator::GtEq | BinaryOperator::LtEq => {
            if left_dtype == &ColumnType::VarChar || right_dtype == &ColumnType::VarChar {
                return false;
            }
            // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
            if let ColumnType::Decimal75(precision, _) = left_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            if let ColumnType::Decimal75(precision, _) = right_dtype {
                if precision.value() > 38 {
                    return false;
                }
            }
            left_dtype.is_numeric() && right_dtype.is_numeric()
                || matches!(
                    (left_dtype, right_dtype),
                    (ColumnType::Boolean, ColumnType::Boolean)
                        | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
                )
        }
        BinaryOperator::Plus => {
            try_add_subtract_column_types(*left_dtype, *right_dtype, BinaryOperator::Plus).is_ok()
        }
        BinaryOperator::Minus => {
            try_add_subtract_column_types(*left_dtype, *right_dtype, BinaryOperator::Minus)
                .is_ok()
        }
        BinaryOperator::Multiply => try_multiply_column_types(*left_dtype, *right_dtype).is_ok(),
        BinaryOperator::Divide => left_dtype.is_numeric() && right_dtype.is_numeric(),
        _ => panic!("Unimplemented binary operator: {binary_operator}"),
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
