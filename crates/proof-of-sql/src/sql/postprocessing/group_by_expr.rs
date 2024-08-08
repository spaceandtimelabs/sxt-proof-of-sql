use bumpalo::Bump;
use proof_of_sql_parser::{intermediate_ast::AliasedResultExpr, Identifier};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A group by expression
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupByExpr {
    /// A list of aggregation column expressions
    result_exprs: Vec<AliasedResultExpr>,

    /// A list of identifiers in the group by clause
    group_by_identifiers: Vec<Identifier>,

    /// A list of aggregation expressions
    agg_expr_pairs: Vec<(AggregationOperator, Expression, Identifier)>,
}

// Get identifiers NOT in aggregate functions
fn get_free_identifiers_from_expr(expr: &Expression) -> HashSet<Identifier> {
    match expr {
        Expression::Column(identifier) => HashSet::from([identifier]),
        Expression::Literal(_) | Expression::AggregateExpr { .. } | Expression::Wildcard => {
            HashSet::new()
        }
        Expression::Binary { op, left, right } => {
            let left_identifiers = get_free_identifiers_from_expr(left);
            let right_identifiers = get_free_identifiers_from_expr(right);
            left_identifiers.union(&right_identifiers);
        }
        Expression::Unary { op, expr } => get_identifiers_from_expr(expr),
    }
}

/// Get aggregate expressions from an expression as well as the remainder
///
/// The idea here is to recursively traverse the expression tree and collect all the aggregation expressions
/// and then label them as new columns post-aggregation and replace them with these new columns so that
/// the post-aggregation expression tree doesn't contain any aggregation expressions and can be simply evaluated.
fn get_aggregate_and_remainder_expressions(
    expr: &Expression,
    agg_count: &mut usize,
) -> (
    Vec<(AggregationOperator, Expression, Identifier)>,
    Expression,
) {
    match expr.expr {
        Expression::Column(_) | Expression::Literal(_) | Expression::Wildcard => {
            (vec![], expr.clone())
        }
        Expression::AggregateExpr { op, expr } => {
            let new_col_id = format!("__col_agg_{}", agg_count).parse().unwrap();
            *agg_count += 1;
            let remainder_expr = Expression::Column(new_col_id);
            (vec![(op, expr.clone(), new_col_id)], remainder_expr)
        }
        Expression::Binary { op, left, right } => {
            let (left_aggs, left_remainder) =
                get_aggregate_and_remainder_expressions(left, agg_count);
            let (right_aggs, right_remainder) =
                get_aggregate_and_remainder_expressions(right, agg_count);
            let aggs = left_aggregate_exprs
                .iter()
                .chain(right_aggregate_exprs.iter())
                .cloned()
                .collect();
            let remainder_expr = Expression::Binary {
                op,
                left: left_remainder,
                right: right_remainder,
            };
            (aggs, remainder_expr)
        }
        Expression::Unary { op, expr } => {
            let (aggs, remainder) = get_aggregate_and_remainder_expressions(expr, agg_count);
            (
                aggs,
                Expression::Unary {
                    op,
                    expr: remainder,
                },
            )
        }
    }
}

/// Given an `AliasedResultExpr`, check if it is legitimate and if so grab the relevant aggregation expression
fn check_and_get_aggregation_and_remainder(
    expr: &AliasedResultExpr,
    group_by_identifiers: &[Identifier],
    agg_count: &mut usize,
) -> PostprocessingResult<(
    Vec<(AggregationOperator, Expression, Identifier)>,
    AliasedResultExpr,
)> {
    let free_identifiers = get_free_identifiers_from_expr(&expr.expr);
    let group_by_identifier_set = group_by_identifiers.iter().collect::<HashSet<_>>();
    if free_identifiers.is_subset(group_by_identifier_set) {
        let (aggs, remainder) = get_aggregate_and_remainder_expressions(&expr.expr, agg_count);
        Ok((
            aggs,
            AliasedResultExpr {
                alias: expr.alias.clone(),
                expr: remainder,
            },
        ))
    } else {
        let diff = free_identifiers
            .difference(group_by_identifiers)
            .into_iter()
            .next()
            .unwrap();
        Err(PostprocessingError::IdentifierNotInAggregationOperatorOrGroupByClause(diff))
    }
}

impl GroupByExpr {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn try_new(
        by_ids: &[Identifier],
        aliased_exprs: &[AliasedResultExpr],
    ) -> PostprocessingResult<Self> {
        let mut agg_count = 0;
        let group_by_identifier_set = by_ids.iter().collect::<HashSet<_>>();
        // Look for aggregation expressions and check for non-aggregation expressions that contain identifiers not in the group by clause
        let (agg_exprs, remainder) = aliased_exprs
            .iter()
            .map(|aliased_expr| -> PostprocessingResult<_> {
                check_and_get_aggregation_and_remainder(aliased_expr, by_ids, &mut agg_count)
            })
            .collect::<PostprocessingResult<Vec<_>>>()?;
        Ok(Self {
            result_exprs: remainder,
            group_by_identifiers: group_by_identifier_set.into_iter().collect::<Vec<_>>(),
            agg_expr_pairs: agg_exprs.into_iter().flatten().collect(),
        })
    }
}

impl<S: Scalar> PostprocessingStep<S> for GroupByExpr {
    /// Apply the group by transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        // First evaluate all the aggregated columns
        let evaluated_columns: Vec<(AggregationOperator, OwnedColumn<S>, Identifier)> = self
            .agg_expr_pairs
            .iter()
            .map(|(agg_op, expr, id)| {
                let evaluated_column = expr.evaluate(&owned_table)?;
                Ok((agg_op, evaluated_column, id))
            })
            .collect::<PostprocessingResult<Vec<_>>>()?;
        // Next actually do the GROUP BY
        let alloc = Bump::new();
        let group_by_columns = self
            .group_by_identifiers
            .iter()
            .map(|id| {
                let column = owned_table
                    .inner_table()
                    .get(id)
                    .ok_or(PostprocessingError::ColumnNotFound(id.to_string()))?;
                Ok(Column::<S>::from_owned_column(column, &alloc))
            })
            .collect::<PostprocessingResult<Vec<_>>>()?;
        let (sum_columns, sum_ids) = evaluated_columns
            .iter()
            .filter(|(agg_op, _owned_col, _id)| agg_op == AggregationOperator::Sum)
            .map(|(agg_op, owned_col, id)| (Column::<S>::from_owned_column(owned_col, &alloc), id))
            .unzip();
        let (max_columns, max_ids) = evaluated_columns
            .iter()
            .filter(|(agg_op, _owned_col, _id)| agg_op == AggregationOperator::Max)
            .map(|(agg_op, owned_col, id)| (Column::<S>::from_owned_column(owned_col, &alloc), id))
            .collect::<Vec<_>>();
        let (min_columns, min_ids) = evaluated_columns
            .iter()
            .filter(|(agg_op, _owned_col, _id)| agg_op == AggregationOperator::Min)
            .map(|(agg_op, owned_col, id)| (Column::<S>::from_owned_column(owned_col, &alloc), id))
            .collect::<Vec<_>>();
        let (count_columns, count_ids) = evaluated_columns
            .iter()
            .filter(|(agg_op, _owned_col, _id)| agg_op == AggregationOperator::Count)
            .map(|(agg_op, owned_col, id)| (Column::<S>::from_owned_column(owned_col, &alloc), id))
            .collect::<Vec<_>>();
        // TODO: Allow a filter
        let selection_column = Column::<S>::from_literal_with_length(
            Literal::Boolean(true),
            owned_table.len(),
            &alloc,
        );
        let aggregation_results = aggregate_columns(
            &alloc,
            &group_by_columns,
            &sum_columns,
            &max_columns,
            &min_columns,
        );
        // Finally do another round of evaluation to get the final result
        // Gather the results into a new OwnedTable
        let group_by_outs = aggregation_results
            .group_by_columns
            .iter()
            .zip(self.group_by_identifiers.iter())
            .map(|(column, id)| (id, OwnedColumn::from(column)))
            .collect::<Vec<_>>();
        let sum_outs = aggregation_results
            .sum_columns
            .iter()
            .zip(sum_ids.iter())
            .map(|(column, id)| (id, OwnedColumn::from(column)))
            .collect::<Vec<_>>();
        let max_outs = aggregation_results
            .max_columns
            .iter()
            .zip(max_ids.iter())
            .map(|(column, id)| (id, OwnedColumn::from(column)))
            .collect::<Vec<_>>();
        let min_outs = aggregation_results
            .min_columns
            .iter()
            .zip(min_ids.iter())
            .map(|(column, id)| (id, OwnedColumn::from(column)))
            .collect::<Vec<_>>();
        let count_outs = aggregation_results
            .count_columns
            .iter()
            .zip(count_ids.iter())
            .map(|(column, id)| (id, OwnedColumn::from(column)))
            .collect::<Vec<_>>();
        let new_owned_table = OwnedTable::try_from_iter(
            group_by_outs
                .iter()
                .chain(sum_outs.iter())
                .chain(max_outs.iter())
                .chain(min_outs.iter())
                .chain(count_outs.iter())
                .map(|(id, column)| (*id, column.clone())),
        )?;
        let result_owned_table =
            OwnedTable::try_from_iter(self.result_exprs.iter().map(|aliased_expr| {
                let column = aliased_expr.expr.evaluate(&new_owned_table)?;
                Ok((aliased_expr.alias.clone(), column))
            }))?;
    }
}
