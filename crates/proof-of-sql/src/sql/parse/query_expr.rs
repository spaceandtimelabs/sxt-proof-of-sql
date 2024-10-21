use super::{EnrichedExpr, FilterExecBuilder, QueryContextBuilder};
use crate::{
    base::{commitment::Commitment, database::SchemaAccessor},
    sql::{
        parse::{ConversionError, ConversionResult},
        postprocessing::{
            GroupByPostprocessing, OrderByPostprocessing, OwnedTablePostprocessing,
            SelectPostprocessing, SlicePostprocessing,
        },
        proof_plans::{DynProofPlan, GroupByExec},
    },
};
use alloc::{fmt, vec, vec::Vec};
use proof_of_sql_parser::{
    intermediate_ast::{
        AliasedResultExpr, BinaryOperator, Expression, SelectResultExpr, SetExpression,
        TableExpression,
    },
    Identifier, SelectStatement,
};
use serde::{Deserialize, Serialize};
use sqlparser::ast::GroupByExpr;

#[derive(PartialEq, Serialize, Deserialize)]
/// A `QueryExpr` represents a Proof of SQL query that can be executed against a database.
/// It consists of a `DynProofPlan` for provable components and a vector of `OwnedTablePostprocessing` for the rest.
pub struct QueryExpr<C: Commitment> {
    proof_expr: DynProofPlan<C>,
    postprocessing: Vec<OwnedTablePostprocessing>,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and postprocessing fields in a readable format.
impl<C: Commitment> fmt::Debug for QueryExpr<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QueryExpr \n[{:#?},\n{:#?}\n]",
            self.proof_expr, self.postprocessing
        )
    }
}

impl<C: Commitment> QueryExpr<C> {
    /// Creates a new `QueryExpr` with the given `DynProofPlan` and `OwnedTablePostprocessing`.
    pub fn new(proof_expr: DynProofPlan<C>, postprocessing: Vec<OwnedTablePostprocessing>) -> Self {
        Self {
            proof_expr,
            postprocessing,
        }
    }

    /// Parse an intermediate AST `SelectStatement` into a `QueryExpr`.
    pub fn try_new(
        ast: SelectStatement,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        let context = match *ast.expr {
            SetExpression::Query {
                result_exprs,
                from,
                where_expr,
                group_by,
            } => QueryContextBuilder::new(schema_accessor)
                .visit_table_expr(&from, default_schema)
                .visit_group_by_exprs(group_by)?
                .visit_result_exprs(result_exprs)?
                .visit_where_expr(where_expr)?
                .visit_order_by_exprs(ast.order_by)
                .visit_slice_expr(ast.slice)
                .build()?,
        };
        let result_aliased_exprs = context.get_aliased_result_exprs()?.to_vec();
        let group_by = context.get_group_by_exprs();

        // Figure out the basic postprocessing steps.
        let mut postprocessing = vec![];
        let order_bys = context.get_order_by_exprs()?;
        if !order_bys.is_empty() {
            postprocessing.push(OwnedTablePostprocessing::new_order_by(
                OrderByPostprocessing::new(order_bys.clone()),
            ));
        }
        if let Some(slice) = context.get_slice_expr() {
            postprocessing.push(OwnedTablePostprocessing::new_slice(
                SlicePostprocessing::new(Some(slice.number_rows), Some(slice.offset_value)),
            ));
        }
        if context.has_agg() {
            if let Some(group_by_expr) = Option::<GroupByExec<C>>::try_from(&context)? {
                Ok(Self {
                    proof_expr: DynProofPlan::GroupBy(group_by_expr),
                    postprocessing,
                })
            } else {
                let raw_enriched_exprs = result_aliased_exprs
                    .iter()
                    .map(|aliased_expr| EnrichedExpr {
                        residue_expression: aliased_expr.clone(),
                        dyn_proof_expr: None,
                    })
                    .collect::<Vec<_>>();
                let filter = FilterExecBuilder::new(context.get_column_mapping())
                    .add_table_expr(*context.get_table_ref())
                    .add_where_expr(context.get_where_expr().clone())?
                    .add_result_columns(&raw_enriched_exprs)
                    .build();

                let group_by_postprocessing =
                    GroupByPostprocessing::try_new(group_by.to_vec(), result_aliased_exprs)?;
                postprocessing.insert(
                    0,
                    OwnedTablePostprocessing::new_group_by(group_by_postprocessing.clone()),
                );
                let remainder_exprs = group_by_postprocessing.remainder_exprs();
                // Check whether we need to do select postprocessing.
                // That is, if any of them is not simply a column reference.
                if remainder_exprs
                    .iter()
                    .any(|expr| expr.try_as_identifier().is_none())
                {
                    postprocessing.insert(
                        1,
                        OwnedTablePostprocessing::new_select(SelectPostprocessing::new(
                            remainder_exprs.to_vec(),
                        )),
                    );
                }
                Ok(Self {
                    proof_expr: DynProofPlan::Filter(filter),
                    postprocessing,
                })
            }
        } else {
            // No group by, so we need to do a filter.
            let column_mapping = context.get_column_mapping();
            let enriched_exprs = result_aliased_exprs
                .iter()
                .map(|aliased_expr| EnrichedExpr::new(aliased_expr.clone(), &column_mapping))
                .collect::<Vec<_>>();
            let select_exprs = enriched_exprs
                .iter()
                .map(|enriched_expr| enriched_expr.residue_expression.clone())
                .collect::<Vec<_>>();
            let filter = FilterExecBuilder::new(context.get_column_mapping())
                .add_table_expr(*context.get_table_ref())
                .add_where_expr(context.get_where_expr().clone())?
                .add_result_columns(&enriched_exprs)
                .build();
            // Check whether we need to do select postprocessing.
            if select_exprs
                .iter()
                .any(|expr| expr.try_as_identifier().is_none())
            {
                postprocessing.insert(
                    0,
                    OwnedTablePostprocessing::new_select(SelectPostprocessing::new(select_exprs)),
                );
            }
            Ok(Self {
                proof_expr: DynProofPlan::Filter(filter),
                postprocessing,
            })
        }
    }

    pub fn try_new_from_sqlparser(
        ast: sqlparser::ast::Query,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        // Extract the main components from the SQLParser AST
        let query_body = match *ast.body {
            sqlparser::ast::SetExpr::Select(select_stmt) => select_stmt,
            _ => return Err(ConversionError::UnsupportedQueryType),
        };

        // Convert SQL AST components (SELECT, WHERE, etc.) into Proof of SQL structures
        let from_clause = query_body
            .from
            .iter()
            .map(|table| Self::convert_sql_table_to_proof_of_sql_table(table))
            .collect::<Result<Vec<_>, _>>()?;

        let result_exprs = query_body
            .projection
            .iter()
            .map(|proj| Self::convert_sql_projection_to_proof_of_sql(proj))
            .collect::<Result<Vec<_>, _>>()?;

        let where_expr = query_body
            .selection
            .map(|expr| Self::convert_sql_expr_to_proof_of_sql(&expr))
            .transpose()?;

        let group_by = match query_body.group_by {
            GroupByExpr::Expressions(exprs) => {
                let mut identifiers = Vec::new();
                for group_by_expr in exprs {
                    match group_by_expr {
                        // If the expression is an identifier, attempt to create a valid Identifier
                        sqlparser::ast::Expr::Identifier(ident) => {
                            match Identifier::new_valid(ident.value.clone()) {
                                Ok(valid_ident) => identifiers.push(valid_ident),
                                Err(e) => {
                                    return Err(ConversionError::ParseError {
                                        error: format!("ParseError: {:?}", e),
                                    });
                                }
                            }
                        }
                        // Handle non-Identifier expressions in GROUP BY clause
                        _ => {
                            return Err(ConversionError::InvalidGroupByColumnRef {
                                column: format!(
                                    "Expected identifier, found expression {:?}",
                                    group_by_expr
                                ),
                            });
                        }
                    }
                }
                identifiers
            }
            GroupByExpr::All => Vec::new(), 
        };

        // Build a QueryContext using the Proof of SQL structures
        let context = QueryContextBuilder::new(schema_accessor)
            .visit_table_expr(&from_clause, default_schema)
            .visit_group_by_exprs(group_by)?
            .visit_result_exprs(result_exprs)?
            .visit_where_expr(where_expr)?
            .visit_order_by_exprs(ast.order_by)
            .visit_slice_expr(ast.slice)
            .build()?;

        // Create and return the QueryExpr with proof_expr and postprocessing
        let enriched_exprs = context.get_aliased_result_exprs()?.to_vec();

        // Build the FilterExec object
        let filter = FilterExecBuilder::new(context.get_column_mapping())
            .add_table_expr(*context.get_table_ref())
            .add_where_expr(context.get_where_expr().clone())?
            .add_result_columns(&enriched_exprs)
            .build();

        // Return the QueryExpr
        Ok(QueryExpr {
            proof_expr: DynProofPlan::Filter(filter),
            postprocessing: vec![],
        })
    }

    /// Immutable access to this query's provable filter expression.
    pub fn proof_expr(&self) -> &DynProofPlan<C> {
        &self.proof_expr
    }

    /// Immutable access to this query's post-proof result transform expression.
    pub fn postprocessing(&self) -> &[OwnedTablePostprocessing] {
        &self.postprocessing
    }

    fn convert_sql_table_to_proof_of_sql_table(
        sql_table: &sqlparser::ast::TableWithJoins,
    ) -> Result<Box<TableExpression>, ConversionError> {
        // Convert SQL table reference to Proof of SQL's TableExpression
        match &sql_table.relation {
            sqlparser::ast::TableFactor::Table { name, .. } => {
                let schema = name
                    .0
                    .get(0)
                    .map(|ident| Identifier::new_valid(ident.value.clone()));
                let table = Identifier::new_valid(name.0.get(1).unwrap().value.clone());
                Ok(Box::new(TableExpression::Named { table, schema }))
            }
            _ => Err(ConversionError::InvalidTable),
        }
    }

    fn convert_sql_projection_to_proof_of_sql(
        projection: &sqlparser::ast::SelectItem,
    ) -> Result<SelectResultExpr, ConversionError> {
        match projection {
            sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                Ok(SelectResultExpr::AliasedResultExpr(AliasedResultExpr {
                    expr: Box::new(convert_sql_expr_to_proof_of_sql(expr)?),
                    alias: Identifier::try_new("alias")?,
                }))
            }
            _ => Err(ConversionError::InvalidProjection),
        }
    }

    fn convert_sql_expr_to_proof_of_sql(
        expr: &sqlparser::ast::Expr,
    ) -> Result<Box<Expression>, ConversionError> {
        match expr {
            sqlparser::ast::Expr::Identifier(ident) => Ok(Box::new(Expression::Column(
                Identifier::new_valid(ident.value.clone()),
            ))),
            sqlparser::ast::Expr::BinaryOp { left, op, right } => {
                let left_expr = convert_sql_expr_to_proof_of_sql(left)?;
                let right_expr = convert_sql_expr_to_proof_of_sql(right)?;
                let op = match op {
                    sqlparser::ast::BinaryOperator::Eq => BinaryOperator::Equal,
                    sqlparser::ast::BinaryOperator::Gt => BinaryOperator::GreaterThanOrEqual,
                    sqlparser::ast::BinaryOperator::Lt => BinaryOperator::LessThanOrEqual,
                    _ => return Err(ConversionError::UnsupportedOperator),
                };
                Ok(Box::new(Expression::Binary {
                    left: left_expr,
                    right: right_expr,
                    op,
                }))
            }
            _ => Err(ConversionError::UnsupportedExpression),
        }
    }
}
