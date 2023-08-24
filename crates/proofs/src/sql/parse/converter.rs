use crate::base::database::{ColumnRef, ColumnType, SchemaAccessor, TableRef};
use crate::base::scalar::ArkScalar;
use crate::sql::ast::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::sql::parse::{ConversionError, ConversionResult, QueryExpr, ResultExprBuilder};
use crate::sql::transform::ResultExpr;
use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, BinaryOperator, Expression, Literal, OrderBy, ResultColumn,
    SelectResultExpr, SetExpression, TableExpression, UnaryOperator,
};
use proofs_sql::{Identifier, ResourceId, SelectStatement};
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Default)]
pub struct Converter {
    /// The current table in context
    current_table: Option<TableRef>,
    /// The result schema of the query
    result_schema: Vec<ResultColumn>,
    /// The aggregation columns appearing in the result schema
    aggregation_columns: Vec<AliasedResultExpr>,
    /// The non-aggregation columns appearing in the result schema
    non_aggregate_columns: Vec<ResultColumn>,
    /// The group by expressions appearing in the query
    group_by_exprs: Vec<(Identifier, Option<Identifier>)>,
}

impl Converter {
    /// Convert an Intermediate AST into a Provable AST
    ///
    /// # Parameters:
    ///
    /// ast: the proper intermediate ast to be converted into a provable ast.
    ///
    /// schema_accessor: this accessor is particularly useful
    ///     to allow us to check if a given column exists in a
    ///     given `schema_table.table_name` as well as check
    ///     its type. We also use it to fetch all columns
    ///     existing in a given `schema_table.table_name`,
    ///     necessary to convert a `select * from T` intermediate ast
    ///     into the provable ast.
    ///
    /// default_schema: in case no schema is specified in the given
    ///     intermediate ast, we use this `default_schema` to
    ///     create the `TableRef`. Otherwise, we use the already
    ///     SelectStatements' schema to create the `TableRef`.
    ///
    /// # Return:
    ///
    /// The provable ast, wrapped inside a parse result.
    pub fn visit_intermediate_ast(
        &mut self,
        ast: &SelectStatement,
        schema_accessor: &dyn SchemaAccessor,
        default_schema: Identifier,
    ) -> ConversionResult<QueryExpr> {
        let filter_expr =
            self.visit_set_expression(ast.expr.deref(), schema_accessor, default_schema)?;

        let result_expr = self.build_result_expr(ast)?;

        Ok(QueryExpr::new(Box::new(filter_expr), Box::new(result_expr)))
    }
}

/// Visit intermediate ast
impl Converter {
    /// Convert a `SetExpression` into a `FilterExpr`
    fn visit_set_expression(
        &mut self,
        expr: &SetExpression,
        schema_accessor: &dyn SchemaAccessor,
        default_schema: Identifier,
    ) -> ConversionResult<FilterExpr> {
        match expr {
            SetExpression::Query {
                result_columns,
                from,
                where_expr,
                group_by,
            } => {
                // we always visit table_expr first, as we need to know the current table name during the next steps.
                let table = self.visit_table_expressions(&from[..], default_schema);

                // gather the non-duplicate references columns from the `group by` and the `result columns`.
                let filter_result_expr_list =
                    self.visit_result_columns(&result_columns[..], group_by, schema_accessor)?;

                // build the filter expression tree out of the `where` clause
                let where_clause = match where_expr {
                    Some(where_expr) => {
                        self.visit_bool_expression(where_expr.deref(), schema_accessor)?
                    }
                    None => Box::new(ConstBoolExpr::new(true)),
                };

                // Populate the group by expressions with their possible respective aliases (when they appear in the result schema)
                //
                // Note: we need to visit the group by expressions after visiting the result columns.
                self.group_by_exprs = self.visit_group_by(group_by)?;

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
    fn visit_table_expression(
        &mut self,
        table_expr: &TableExpression,
        default_schema: Identifier,
    ) -> TableExpr {
        match table_expr {
            TableExpression::Named { table, schema } => {
                let schema = schema.unwrap_or(default_schema);

                let table_ref = TableRef::new(ResourceId::new(schema, *table));

                self.current_table = Some(table_ref);

                TableExpr { table_ref }
            }
        }
    }

    /// Convert a `TableExpression slice` into a `TableExpr`
    fn visit_table_expressions(
        &mut self,
        table_exprs: &[Box<TableExpression>],
        default_schema: Identifier,
    ) -> TableExpr {
        assert!(table_exprs.len() == 1);

        self.visit_table_expression(table_exprs[0].deref(), default_schema)
    }
}

/// Build result expr
impl Converter {
    fn build_result_expr(&self, ast: &SelectStatement) -> ConversionResult<ResultExpr> {
        self.check_order_by(&ast.order_by[..])?;

        let result_schema = self
            .result_schema
            .iter()
            .map(|col| AliasedResultExpr {
                expr: proofs_sql::intermediate_ast::ResultExpr::NonAgg(Box::new(
                    Expression::Column(col.name),
                )),
                alias: col.alias,
            })
            .collect();

        Ok(ResultExprBuilder::default()
            .add_group_by(
                self.group_by_exprs.to_vec(),
                self.aggregation_columns.to_vec(),
            )
            .add_select(result_schema)
            .add_order_by(ast.order_by.to_vec())
            .add_slice(&ast.slice)
            .build())
    }
}

// Group By
impl Converter {
    /// Convert a `GroupBy` into a `Vec<(Identifier, Option<Identifier>)>`
    ///
    /// Group by names are always propagated to the first element of the tuple.
    /// Thus they always need to reference a valid column name existing in the table.
    /// When the group by name is part of the result schema, the second element
    /// is set as the respective result alias name. Otherwise, it is set to `None`.
    fn visit_group_by(
        &self,
        group_by_exprs: &[Identifier],
    ) -> ConversionResult<Vec<(Identifier, Option<Identifier>)>> {
        if group_by_exprs.is_empty() {
            return Ok(Vec::new());
        }

        // We need to remap the group by column names so that we can easily know if
        // they are part of the result schema.
        let mut transform_group_by_exprs = Vec::new();

        // We need to add the group by columns that are part of the result schema.
        for col in self.non_aggregate_columns.iter() {
            // We need to check that each non aggregated result column is referenced in the group by clause.
            if group_by_exprs.iter().any(|group_by| *group_by == col.name) {
                // note: `col.alias` here implies that this group by will appear in the result schema using `col.alias`.
                transform_group_by_exprs.push((col.name, Some(col.alias)));
            } else {
                return Err(ConversionError::InvalidGroupByResultColumnError);
            }
        }

        // We need to add the group by columns that are not part of the result schema.
        for group_by in group_by_exprs.iter() {
            if !self
                .non_aggregate_columns
                .iter()
                .any(|col| col.name == *group_by)
            {
                // note: `None` here implies that the this group by will be filtered out of the result schema.
                transform_group_by_exprs.push((*group_by, None))
            }
        }

        Ok(transform_group_by_exprs)
    }
}

// Order By
impl Converter {
    /// Check that each order by expression is associated with an existing column alias.
    /// Order by values associated with result column names are not allowed.
    fn check_order_by(&self, by_exprs: &[OrderBy]) -> ConversionResult<()> {
        for by_expr in by_exprs {
            self.result_schema
                .iter()
                .find(|col| col.alias == by_expr.expr)
                .ok_or(ConversionError::InvalidOrderByError(
                    by_expr.expr.as_str().to_string(),
                ))?;
        }
        Ok(())
    }
}

/// Result expression
impl Converter {
    fn get_table_schema(
        &self,
        schema_accessor: &dyn SchemaAccessor,
    ) -> Vec<(Identifier, ColumnType)> {
        let current_table = *self
            .current_table
            .as_ref()
            .expect("Some table should've already been processed at this point");

        schema_accessor.lookup_schema(current_table)
    }

    fn visit_result_column_all(&self, schema_accessor: &dyn SchemaAccessor) -> Vec<ResultColumn> {
        let table_schema = self.get_table_schema(schema_accessor);

        table_schema
            .into_iter()
            .map(|(name, _)| ResultColumn { name, alias: name })
            .collect()
    }

    fn visit_result_aggregation_expression(
        &mut self,
        agg_expr: &AggExpr,
        group_by_exprs: &[Identifier],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<(AggExpr, Identifier)> {
        // We can't aggregate without specifying a group by column
        if group_by_exprs.is_empty() {
            return Err(ConversionError::MissingGroupByError);
        }

        match &agg_expr {
            AggExpr::Max(expr) | AggExpr::Min(expr) | AggExpr::Sum(expr) => {
                let column = match expr.deref() {
                    Expression::Column(column) => {
                        self.visit_column_identifier(*column, schema_accessor)?
                    }
                    _ => {
                        panic!("Unsupported expression type. Must be rejected at the parser phase")
                    }
                };

                // We only support max aggregation on numeric columns
                if column.column_type() != &ColumnType::BigInt {
                    return Err(ConversionError::NonNumericColumnAggregation("max"));
                }

                Ok((agg_expr.clone(), column.column_id()))
            }
            AggExpr::Count(expr) => {
                let column = match expr.deref() {
                    Expression::Column(column) => {
                        self.visit_column_identifier(*column, schema_accessor)?
                    }
                    _ => {
                        panic!("Unsupported expression type. Must be rejected at the parser phase")
                    }
                };

                Ok((agg_expr.clone(), column.column_id()))
            }
            AggExpr::CountALL => {
                // Here we could use any column available in the table.
                // But due to efficiency reasons, we pick the first
                // column in the group by clause as it'll already
                // be available in the result record batch.
                let name: Identifier = group_by_exprs[0];
                Ok((AggExpr::Count(Box::new(Expression::Column(name))), name))
            }
        }
    }

    fn check_result_columns_has_unique_aliases(
        &self,
        result_columns: &[ResultColumn],
    ) -> ConversionResult<()> {
        let mut aliases_set = HashSet::new();
        for column in result_columns {
            let alias = column.alias;

            // we don't allow duplicate aliases
            if !aliases_set.insert(alias) {
                return Err(ConversionError::DuplicateColumnAlias(
                    alias.name().to_string(),
                ));
            }
        }

        Ok(())
    }

    fn visit_result_column_expressions(
        &mut self,
        result_columns: &[SelectResultExpr],
        group_by_exprs: &[Identifier],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<ResultColumn>> {
        self.aggregation_columns = vec![];
        self.non_aggregate_columns = vec![];

        let result_columns = result_columns
            .iter()
            .map(|result_column| match result_column {
                SelectResultExpr::ALL => {
                    let result_column_all = self.visit_result_column_all(schema_accessor);

                    // We need to know which group by expressions will be part of
                    // select result clause. To help with that, we need to keep
                    // track of the non-aggregate columns.
                    self.non_aggregate_columns
                        .extend_from_slice(&result_column_all);

                    Ok(result_column_all)
                }
                SelectResultExpr::AliasedResultExpr(aliased_expr) => {
                    let result_column = match &aliased_expr.expr {
                        proofs_sql::intermediate_ast::ResultExpr::NonAgg(column_expr) => {
                            let column = match column_expr.deref() {
                                Expression::Column(column) => {
                                    self.visit_column_identifier(*column, schema_accessor)?;
                                    column
                                }
                                _ => panic!("Unsupported expression type. Must be rejected at the parser phase"),
                            };

                            let result_column = ResultColumn {name: *column, alias: aliased_expr.alias};

                            // We need to know which group by expressions will be part of
                            // select result clause. To help with that, we need to keep
                            // track of the non-aggregate columns.
                            self.non_aggregate_columns.push(result_column);

                            result_column
                        }
                        proofs_sql::intermediate_ast::ResultExpr::Agg(agg_expr) => {
                            let (agg_expr, column) = self.visit_result_aggregation_expression(
                                agg_expr,
                                group_by_exprs,
                                schema_accessor,
                            )?;

                            let result_column = ResultColumn { name: column, alias: aliased_expr.alias};
                            let agg_aliased_expr = AliasedResultExpr { expr: proofs_sql::intermediate_ast::ResultExpr::Agg(agg_expr), alias: aliased_expr.alias};
                            // We need to keep track of the aggregate expressions
                            // as they are used to build the GroupByExpr node.
                            self.aggregation_columns.push(agg_aliased_expr);

                            result_column
                        }
                    };

                    Ok(vec![result_column])
                }
            })
            .collect::<ConversionResult<Vec<_>>>()?;

        let result_columns = result_columns.into_iter().flatten().collect::<Vec<_>>();

        self.check_result_columns_has_unique_aliases(&result_columns)?;

        Ok(result_columns)
    }

    /// Convert a `ResultColumnExpr slice` into a `Vec<FilterResultExpr>`
    fn visit_result_columns(
        &mut self,
        result_columns: &[SelectResultExpr],
        group_by_exprs: &[Identifier],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<FilterResultExpr>> {
        assert!(!result_columns.is_empty());

        // Gather all the result columns
        self.result_schema =
            self.visit_result_column_expressions(result_columns, group_by_exprs, schema_accessor)?;

        // Get the HashSet of all column names in the result schema
        //
        // Note: we chain the group by expressions as their respective
        // columns need to be available in the result record batch
        // during post-processing.
        let non_duplicate_result_columns = self
            .result_schema
            .iter()
            .map(|col| &col.name)
            .chain(group_by_exprs.iter())
            .collect::<HashSet<_>>();

        // Convert the hash_set to a vector and sort it
        let mut non_duplicate_result_columns =
            non_duplicate_result_columns.into_iter().collect::<Vec<_>>();

        // Sorting is required to make the relative order of the columns deterministic
        // `Unstable sort` is used as it's more efficient than `Sort`.
        non_duplicate_result_columns.sort_unstable();

        // Convert the column names vector into a vector of FilterResultExpr
        let non_duplicate_filter_result_columns = non_duplicate_result_columns
            .into_iter()
            .map(|name| {
                // Note: during the `visit_column_identifier`, we also check
                // if the column name `*name` is present in the table schema.
                // This check ensures that either result columns or group by
                // expressions have valid column names.
                Ok(FilterResultExpr::new(
                    self.visit_column_identifier(*name, schema_accessor)?,
                ))
            })
            .collect::<ConversionResult<Vec<_>>>()?;

        Ok(non_duplicate_filter_result_columns)
    }
}

/// Where expression
impl Converter {
    /// Convert an `Expression` into a BoolExpr
    fn visit_bool_expression(
        &self,
        expression: &Expression,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Box<dyn BoolExpr>> {
        match expression {
            Expression::Unary { op, expr } => match &op {
                UnaryOperator::Not => Ok(Box::new(NotExpr::new(
                    self.visit_bool_expression(expr.deref(), schema_accessor)?,
                ))),
            },
            Expression::Binary { op, left, right } => match op {
                BinaryOperator::And => {
                    let left = self.visit_bool_expression(left.deref(), schema_accessor)?;
                    let right = self.visit_bool_expression(right.deref(), schema_accessor)?;
                    Ok(Box::new(AndExpr::new(left, right)))
                }
                BinaryOperator::Or => {
                    let left = self.visit_bool_expression(left.deref(), schema_accessor)?;
                    let right = self.visit_bool_expression(right.deref(), schema_accessor)?;
                    Ok(Box::new(OrExpr::new(left, right)))
                }
                BinaryOperator::Equal => {
                    self.visit_equal_expression(*op, left.deref(), right.deref(), schema_accessor)
                }
            },
            _ => panic!("Unsupported expression type. Must be rejected at the parser phase"),
        }
    }

    /// Convert an `Expression` into an EqualsExpr
    fn visit_equal_expression(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Box<dyn BoolExpr>> {
        assert_eq!(op, BinaryOperator::Equal);

        let (literal, literal_dtype) = match right {
            Expression::Literal(literal) => self.visit_literal(literal),
            _ => panic!("Unsupported expression type. Must be rejected at the parser phase"),
        };
        let column_ref = match left {
            Expression::Column(column) => self.visit_column_identifier(*column, schema_accessor)?,
            _ => panic!("Unsupported expression type. Must be rejected at the parser phase"),
        };

        if *column_ref.column_type() != literal_dtype {
            return Err(ConversionError::MismatchTypeError(format!(
                "Literal \"{:?}\" has type {:?} but column \"{:?}\" from table \"{:?}\" has type {:?}",
                literal,
                literal_dtype,
                column_ref.column_id(),
                column_ref.table_ref(),
                column_ref.column_type()
            )));
        }

        Ok(Box::new(EqualsExpr::new(column_ref, literal)))
    }
}

/// Tokens (literals and id's)
impl Converter {
    fn visit_literal(&self, literal: &Literal) -> (ArkScalar, ColumnType) {
        match literal {
            Literal::BigInt(val) => (val.into(), ColumnType::BigInt),
            Literal::VarChar(val) => (val.into(), ColumnType::VarChar),
        }
    }

    /// Convert a `Name` into an identifier string (i.e. a string)
    fn visit_column_identifier(
        &self,
        column_name: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<ColumnRef> {
        let current_table = *self
            .current_table
            .as_ref()
            .expect("Some table should've already been processed at this point");

        let column_type = schema_accessor.lookup_column(current_table, column_name);
        let column_type = column_type.ok_or_else(|| {
            ConversionError::MissingColumnError(format!(
                "Column \"{}\" is not found in table \"{}\"",
                column_name,
                current_table.table_id()
            ))
        })?;

        Ok(ColumnRef::new(current_table, column_name, column_type))
    }
}
