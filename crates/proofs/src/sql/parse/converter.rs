use crate::base::database::{ColumnRef, ColumnType, SchemaAccessor, TableRef};
use crate::sql::ast::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::sql::parse::{ConversionError, ConversionResult, QueryExpr, ResultExprBuilder};
use crate::sql::transform::ResultExpr;

use crate::base::polynomial::ArkScalar;
use proofs_sql::intermediate_ast::{
    AggExpr, Expression, Literal, OrderBy, ResultColumn, ResultColumnExpr, SetExpression,
    TableExpression,
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
    aggregation_columns: Vec<AggExpr>,
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
                columns,
                from,
                where_expr,
                group_by,
            } => {
                // we always visit table_expr first, as we need to know the current table name during the next steps.
                let table = self.visit_table_expressions(&from[..], default_schema);

                // gather the non-duplicate references columns from the `group by` and the `result columns`.
                let filter_result_expr_list =
                    self.visit_result_columns(&columns[..], group_by, schema_accessor)?;

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

        let mut result_expr_builder = ResultExprBuilder::default();

        if self.group_by_exprs.is_empty() && ast.order_by.is_empty() && ast.slice.is_none() {
            // we need to apply a projection if there is no transformations
            return Ok(ResultExpr::new_with_result_schema(
                self.result_schema.to_vec(),
            ));
        }

        if self.group_by_exprs.is_empty() {
            // select must be applied before order by as
            // it references aliases defined in the select clause
            result_expr_builder.add_select(self.result_schema.to_vec());
        } else {
            result_expr_builder.add_group_by(
                self.group_by_exprs.to_vec(),
                self.aggregation_columns.to_vec(),
            );

            // Group by modifies the result schema order and name so that
            // only aliases exist in the final lazy frame.
            //
            // Therefore, we need to re-map the select expression to reflect
            // the group by changes.
            let result_schema = self
                .result_schema
                .iter()
                .map(|col| ResultColumn {
                    name: col.alias,
                    alias: col.alias,
                })
                .collect::<Vec<_>>();

            result_expr_builder.add_select(result_schema);
        }

        result_expr_builder.add_order_by(ast.order_by.to_vec());

        if let Some(slice) = &ast.slice {
            result_expr_builder.add_slice(slice.number_rows, slice.offset_value);
        }

        Ok(result_expr_builder.build())
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
    ) -> ConversionResult<Vec<ResultColumn>> {
        // We can't aggregate without specifying a group by column
        if group_by_exprs.is_empty() {
            return Err(ConversionError::MissingGroupByError);
        }

        match &agg_expr {
            AggExpr::Max(result_column) => {
                let column = self.visit_column_identifier(result_column.name, schema_accessor)?;

                // We only support max aggregation on numeric columns
                if column.column_type() != &ColumnType::BigInt {
                    return Err(ConversionError::NonNumericColumnAggregation("max"));
                }

                Ok(vec![result_column.clone()])
            }
            AggExpr::Min(result_column) => {
                let column = self.visit_column_identifier(result_column.name, schema_accessor)?;

                // We only support min aggregation on numeric columns
                if column.column_type() != &ColumnType::BigInt {
                    return Err(ConversionError::NonNumericColumnAggregation("min"));
                }

                Ok(vec![result_column.clone()])
            }
            AggExpr::Sum(result_column) => {
                let column = self.visit_column_identifier(result_column.name, schema_accessor)?;

                // We only support sum aggregation on numeric columns
                if column.column_type() != &ColumnType::BigInt {
                    return Err(ConversionError::NonNumericColumnAggregation("sum"));
                }

                Ok(vec![result_column.clone()])
            }
            AggExpr::Count(result_column) => Ok(vec![result_column.clone()]),
            AggExpr::CountAll(alias) => {
                // Here we could use any column available in the table.
                // But due to efficiency reasons, we pick the first
                // column in the group by clause as it'll already
                // be available in the result record batch.
                let name = group_by_exprs[0];

                let result_column = ResultColumn {
                    name,
                    alias: *alias,
                };

                Ok(vec![result_column])
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
        result_columns: &[ResultColumnExpr],
        group_by_exprs: &[Identifier],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<ResultColumn>> {
        self.aggregation_columns = vec![];
        self.non_aggregate_columns = vec![];

        let result_columns = result_columns
            .iter()
            .map(|result_column| match result_column {
                ResultColumnExpr::AllColumns => {
                    let result_column_all = self.visit_result_column_all(schema_accessor);

                    // we need to keep track of the non-aggregate columns
                    // as they are used to build the GroupByExpr node.
                    self.non_aggregate_columns
                        .extend_from_slice(&result_column_all);

                    Ok(result_column_all)
                }
                ResultColumnExpr::SimpleColumn(result_column) => {
                    // we need to keep track of the non-aggregate columns
                    // as they are used to build the GroupByExpr node.
                    self.non_aggregate_columns.push(result_column.clone());

                    Ok(vec![result_column.clone()])
                }
                ResultColumnExpr::AggColumn(agg_expr) => {
                    // We need to keep track of the aggregate expressions
                    // as they are used to build the GroupByExpr node.
                    self.aggregation_columns.push(agg_expr.clone());

                    self.visit_result_aggregation_expression(
                        agg_expr,
                        group_by_exprs,
                        schema_accessor,
                    )
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
        result_columns: &[ResultColumnExpr],
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

            Expression::Equal { left, right } => {
                self.visit_equals_expression(*left, right, schema_accessor)
            }
        }
    }

    /// Convert an `Expression` into an EqualsExpr
    fn visit_equals_expression(
        &self,
        left: Identifier,
        right: &Literal,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Box<dyn BoolExpr>> {
        let (scalar, dtype) = self.visit_literal(right.deref());
        let column_ref = self.visit_column_identifier(left, schema_accessor)?;

        if *column_ref.column_type() != dtype {
            return Err(ConversionError::MismatchTypeError(format!(
                "Literal \"{:?}\" has type {:?} but column \"{:?}\" from table \"{:?}\" has type {:?}",
                right.deref(),
                dtype,
                column_ref.column_id(),
                column_ref.table_ref(),
                column_ref.column_type()
            )));
        }

        Ok(Box::new(EqualsExpr::new(column_ref, scalar)))
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
