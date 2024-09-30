use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{group_by_util::aggregate_columns, Column, OwnedColumn, OwnedTable},
    map::{indexmap, IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};
use bumpalo::Bump;
use itertools::{izip, Itertools};
use proof_of_sql_parser::{
    intermediate_ast::{AggregationOperator, AliasedResultExpr, Expression},
    Identifier,
};
use serde::{Deserialize, Serialize};

/// A group by expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupByPostprocessing {
    /// A list of `AliasedResultExpr` that exclusively use identifiers in the group by clause or results of aggregation expressions
    remainder_exprs: Vec<AliasedResultExpr>,

    /// A list of identifiers in the group by clause
    group_by_identifiers: Vec<Identifier>,

    /// A list of aggregation expressions
    aggregation_exprs: Vec<(AggregationOperator, Expression, Identifier)>,
}

/// Check whether multiple layers of aggregation exist within the same GROUP BY clause
/// since this is not allowed in SQL
///
/// If the context is within an aggregation function, then any aggregation function is considered nested.
/// Otherwise we need two layers of aggregation functions to be nested.
fn contains_nested_aggregation(expr: &Expression, is_agg: bool) -> bool {
    match expr {
        Expression::Column(_) | Expression::Literal(_) | Expression::Wildcard => false,
        Expression::Aggregation { expr, .. } => is_agg || contains_nested_aggregation(expr, true),
        Expression::Binary { left, right, .. } => {
            contains_nested_aggregation(left, is_agg) || contains_nested_aggregation(right, is_agg)
        }
        Expression::Unary { expr, .. } => contains_nested_aggregation(expr, is_agg),
    }
}

/// Get identifiers NOT in aggregate functions
fn get_free_identifiers_from_expr(expr: &Expression) -> IndexSet<Identifier> {
    match expr {
        Expression::Column(identifier) => IndexSet::from_iter([*identifier]),
        Expression::Literal(_) | Expression::Aggregation { .. } | Expression::Wildcard => {
            IndexSet::default()
        }
        Expression::Binary { left, right, .. } => {
            let mut left_identifiers = get_free_identifiers_from_expr(left);
            let right_identifiers = get_free_identifiers_from_expr(right);
            left_identifiers.extend(right_identifiers);
            left_identifiers
        }
        Expression::Unary { expr, .. } => get_free_identifiers_from_expr(expr),
    }
}

/// Get aggregate expressions from an expression as well as the remainder
///
/// The idea here is to recursively traverse the expression tree and collect all the aggregation expressions
/// and then label them as new columns post-aggregation and replace them with these new columns so that
/// the post-aggregation expression tree doesn't contain any aggregation expressions and can be simply evaluated.
fn get_aggregate_and_remainder_expressions(
    expr: Expression,
    aggregation_expr_map: &mut IndexMap<(AggregationOperator, Expression), Identifier>,
) -> Expression {
    match expr {
        Expression::Column(_) | Expression::Literal(_) | Expression::Wildcard => expr,
        Expression::Aggregation { op, expr } => {
            let key = (op, (*expr));
            if !aggregation_expr_map.contains_key(&key) {
                let new_col_id = format!("__col_agg_{}", aggregation_expr_map.len())
                    .parse()
                    .unwrap();
                aggregation_expr_map.insert(key, new_col_id);
                Expression::Column(new_col_id)
            } else {
                Expression::Column(*aggregation_expr_map.get(&key).unwrap())
            }
        }
        Expression::Binary { op, left, right } => {
            let left_remainder =
                get_aggregate_and_remainder_expressions(*left, aggregation_expr_map);
            let right_remainder =
                get_aggregate_and_remainder_expressions(*right, aggregation_expr_map);
            Expression::Binary {
                op,
                left: Box::new(left_remainder),
                right: Box::new(right_remainder),
            }
        }
        Expression::Unary { op, expr } => {
            let remainder = get_aggregate_and_remainder_expressions(*expr, aggregation_expr_map);
            Expression::Unary {
                op,
                expr: Box::new(remainder),
            }
        }
    }
}

/// Given an `AliasedResultExpr`, check if it is legitimate and if so grab the relevant aggregation expression
fn check_and_get_aggregation_and_remainder(
    expr: AliasedResultExpr,
    group_by_identifiers: &[Identifier],
    aggregation_expr_map: &mut IndexMap<(AggregationOperator, Expression), Identifier>,
) -> PostprocessingResult<AliasedResultExpr> {
    let free_identifiers = get_free_identifiers_from_expr(&expr.expr);
    let group_by_identifier_set = group_by_identifiers
        .iter()
        .cloned()
        .collect::<IndexSet<_>>();
    if contains_nested_aggregation(&expr.expr, false) {
        return Err(PostprocessingError::NestedAggregationInGroupByClause {
            error: format!("Nested aggregations found {:?}", expr.expr),
        });
    }
    if free_identifiers.is_subset(&group_by_identifier_set) {
        let remainder = get_aggregate_and_remainder_expressions(*expr.expr, aggregation_expr_map);
        Ok(AliasedResultExpr {
            alias: expr.alias,
            expr: Box::new(remainder),
        })
    } else {
        let diff = free_identifiers
            .difference(&group_by_identifier_set)
            .next()
            .unwrap();
        Err(
            PostprocessingError::IdentifierNotInAggregationOperatorOrGroupByClause {
                column: *diff,
            },
        )
    }
}

impl GroupByPostprocessing {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn try_new(
        by_ids: Vec<Identifier>,
        aliased_exprs: Vec<AliasedResultExpr>,
    ) -> PostprocessingResult<Self> {
        let mut aggregation_expr_map: IndexMap<(AggregationOperator, Expression), Identifier> =
            IndexMap::default();
        // Look for aggregation expressions and check for non-aggregation expressions that contain identifiers not in the group by clause
        let remainder_exprs: Vec<AliasedResultExpr> = aliased_exprs
            .into_iter()
            .map(|aliased_expr| -> PostprocessingResult<_> {
                check_and_get_aggregation_and_remainder(
                    aliased_expr,
                    &by_ids,
                    &mut aggregation_expr_map,
                )
            })
            .collect::<PostprocessingResult<Vec<AliasedResultExpr>>>()?;
        let group_by_identifiers = Vec::from_iter(IndexSet::from_iter(by_ids));
        Ok(Self {
            remainder_exprs,
            group_by_identifiers,
            aggregation_exprs: aggregation_expr_map
                .into_iter()
                .map(|((op, expr), id)| (op, expr, id))
                .collect(),
        })
    }

    /// Get group by identifiers
    pub fn group_by(&self) -> &[Identifier] {
        &self.group_by_identifiers
    }

    /// Get remainder expressions for SELECT
    pub fn remainder_exprs(&self) -> &[AliasedResultExpr] {
        &self.remainder_exprs
    }

    /// Get aggregation expressions
    pub fn aggregation_exprs(&self) -> &[(AggregationOperator, Expression, Identifier)] {
        &self.aggregation_exprs
    }
}

impl<S: Scalar> PostprocessingStep<S> for GroupByPostprocessing {
    /// Apply the group by transformation to the given `OwnedTable`.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        // First evaluate all the aggregated columns
        let alloc = Bump::new();
        let evaluated_columns = self
            .aggregation_exprs
            .iter()
            .map(|(agg_op, expr, id)| -> PostprocessingResult<_> {
                let evaluated_owned_column = owned_table.evaluate(expr)?;
                Ok((*agg_op, (*id, evaluated_owned_column)))
            })
            .process_results(|iter| {
                iter.fold(
                    IndexMap::<_, Vec<_>>::default(),
                    |mut lookup, (key, val)| {
                        lookup.entry(key).or_default().push(val);
                        lookup
                    },
                )
            })?;
        // Next actually do the GROUP BY
        let group_by_ins = self
            .group_by_identifiers
            .iter()
            .map(|id| {
                let column = owned_table.inner_table().get(id).ok_or(
                    PostprocessingError::ColumnNotFound {
                        column: id.to_string(),
                    },
                )?;
                Ok(Column::<S>::from_owned_column(column, &alloc))
            })
            .collect::<PostprocessingResult<Vec<_>>>()?;
        // TODO: Allow a filter
        let selection_in = vec![true; owned_table.num_rows()];
        let (sum_ids, sum_ins): (Vec<_>, Vec<_>) = evaluated_columns
            .get(&AggregationOperator::Sum)
            .map(|tuple| {
                tuple
                    .iter()
                    .map(|(id, c)| (*id, Column::<S>::from_owned_column(c, &alloc)))
                    .unzip()
            })
            .unwrap_or((vec![], vec![]));
        let (max_ids, max_ins): (Vec<_>, Vec<_>) = evaluated_columns
            .get(&AggregationOperator::Max)
            .map(|tuple| {
                tuple
                    .iter()
                    .map(|(id, c)| (*id, Column::<S>::from_owned_column(c, &alloc)))
                    .unzip()
            })
            .unwrap_or((vec![], vec![]));
        let (min_ids, min_ins): (Vec<_>, Vec<_>) = evaluated_columns
            .get(&AggregationOperator::Min)
            .map(|tuple| {
                tuple
                    .iter()
                    .map(|(id, c)| (*id, Column::<S>::from_owned_column(c, &alloc)))
                    .unzip()
            })
            .unwrap_or((vec![], vec![]));
        let aggregation_results = aggregate_columns(
            &alloc,
            &group_by_ins,
            &sum_ins,
            &max_ins,
            &min_ins,
            &selection_in,
        )?;
        // Finally do another round of evaluation to get the final result
        // Gather the results into a new OwnedTable
        let group_by_outs = aggregation_results
            .group_by_columns
            .iter()
            .zip(self.group_by_identifiers.iter())
            .map(|(column, id)| Ok((*id, OwnedColumn::from(column))));
        let sum_outs =
            izip!(aggregation_results.sum_columns, sum_ids, sum_ins,).map(|(c_out, id, c_in)| {
                Ok((
                    id,
                    OwnedColumn::try_from_scalars(c_out, c_in.column_type())?,
                ))
            });
        let max_outs =
            izip!(aggregation_results.max_columns, max_ids, max_ins,).map(|(c_out, id, c_in)| {
                Ok((
                    id,
                    OwnedColumn::try_from_option_scalars(c_out, c_in.column_type())?,
                ))
            });
        let min_outs =
            izip!(aggregation_results.min_columns, min_ids, min_ins,).map(|(c_out, id, c_in)| {
                Ok((
                    id,
                    OwnedColumn::try_from_option_scalars(c_out, c_in.column_type())?,
                ))
            });
        //TODO: When we have NULLs we need to differentiate between count(1) and count(expression)
        let count_column = OwnedColumn::BigInt(aggregation_results.count_column.to_vec());
        let count_outs = evaluated_columns
            .get(&AggregationOperator::Count)
            .into_iter()
            .flatten()
            .map(|(id, _)| -> PostprocessingResult<_> { Ok((*id, count_column.clone())) });
        let new_owned_table: OwnedTable<S> = group_by_outs
            .into_iter()
            .chain(sum_outs)
            .chain(max_outs)
            .chain(min_outs)
            .chain(count_outs)
            .process_results(|iter| OwnedTable::try_from_iter(iter))??;
        // If there are no columns at all we need to have the count column so that we can handle
        // queries such as `SELECT 1 FROM table`
        let target_table = if new_owned_table.is_empty() {
            OwnedTable::try_new(indexmap! {"__count__".parse().unwrap() => count_column})?
        } else {
            new_owned_table
        };
        let res = self
            .remainder_exprs
            .iter()
            .map(|aliased_expr| -> PostprocessingResult<_> {
                let column = target_table.evaluate(&aliased_expr.expr)?;
                Ok((aliased_expr.alias, column))
            })
            .process_results(|iter| OwnedTable::try_from_iter(iter))??;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proof_of_sql_parser::utility::*;

    #[test]
    fn we_can_detect_nested_aggregation() {
        // SUM(SUM(a))
        let expr = sum(sum(col("a")));
        assert!(contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // MAX(a) + SUM(b)
        let expr = add(max(col("a")), sum(col("b")));
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + SUM(b)
        let expr = add(col("a"), sum(col("b")));
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // SUM(a) + b - SUM(2 * c)
        let expr = sub(add(sum(col("a")), col("b")), sum(mul(lit(2), col("c"))));
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + COUNT(SUM(a))
        let expr = add(col("a"), count(sum(col("a"))));
        assert!(contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + b + 1
        let expr = add(add(col("a"), col("b")), lit(1));
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(!contains_nested_aggregation(&expr, true));
    }

    #[test]
    fn we_can_get_free_identifiers_from_expr() {
        // Literal
        let expr = lit("Not an identifier");
        let expected: IndexSet<Identifier> = IndexSet::default();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // a + b + 1
        let expr = add(add(col("a"), col("b")), lit(1));
        let expected: IndexSet<Identifier> = [ident("a"), ident("b")].iter().cloned().collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // ! (a == b || c >= a)
        let expr = not(or(equal(col("a"), col("b")), ge(col("c"), col("a"))));
        let expected: IndexSet<Identifier> = [ident("a"), ident("b"), ident("c")]
            .iter()
            .cloned()
            .collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // SUM(a + b) * 2
        let expr = mul(sum(add(col("a"), col("b"))), lit(2));
        let expected: IndexSet<Identifier> = IndexSet::default();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // (COUNT(a + b) + c) * d
        let expr = mul(add(count(add(col("a"), col("b"))), col("c")), col("d"));
        let expected: IndexSet<Identifier> = [ident("c"), ident("d")].iter().cloned().collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_get_aggregate_and_remainder_expressions() {
        let mut aggregation_expr_map: IndexMap<(AggregationOperator, Expression), Identifier> =
            IndexMap::default();
        // SUM(a) + b
        let expr = add(sum(col("a")), col("b"));
        let remainder_expr =
            get_aggregate_and_remainder_expressions(*expr, &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&(AggregationOperator::Sum, *col("a"))],
            ident("__col_agg_0")
        );
        assert_eq!(remainder_expr, *add(col("__col_agg_0"), col("b")));
        assert_eq!(aggregation_expr_map.len(), 1);

        // SUM(a) + SUM(b)
        let expr = add(sum(col("a")), sum(col("b")));
        let remainder_expr =
            get_aggregate_and_remainder_expressions(*expr, &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&(AggregationOperator::Sum, *col("a"))],
            ident("__col_agg_0")
        );
        assert_eq!(
            aggregation_expr_map[&(AggregationOperator::Sum, *col("b"))],
            ident("__col_agg_1")
        );
        assert_eq!(remainder_expr, *add(col("__col_agg_0"), col("__col_agg_1")));
        assert_eq!(aggregation_expr_map.len(), 2);

        // MAX(a + 1) + MIN(2 * b - 4) + c
        let expr = add(
            add(
                max(col("a") + lit(1)),
                min(sub(mul(lit(2), col("b")), lit(4))),
            ),
            col("c"),
        );
        let remainder_expr =
            get_aggregate_and_remainder_expressions(*expr, &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&(AggregationOperator::Max, *add(col("a"), lit(1)))],
            ident("__col_agg_2")
        );
        assert_eq!(
            aggregation_expr_map[&(
                AggregationOperator::Min,
                *sub(mul(lit(2), col("b")), lit(4))
            )],
            ident("__col_agg_3")
        );
        assert_eq!(
            remainder_expr,
            *add(add(col("__col_agg_2"), col("__col_agg_3")), col("c"))
        );
        assert_eq!(aggregation_expr_map.len(), 4);

        // COUNT(2 * a) * 2 + SUM(b) + 1
        let expr = add(
            add(mul(count(mul(lit(2), col("a"))), lit(2)), sum(col("b"))),
            lit(1),
        );
        let remainder_expr =
            get_aggregate_and_remainder_expressions(*expr, &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&(AggregationOperator::Count, *mul(lit(2), col("a")))],
            ident("__col_agg_4")
        );
        assert_eq!(
            remainder_expr,
            *add(
                add(mul(col("__col_agg_4"), lit(2)), col("__col_agg_1")),
                lit(1)
            )
        );
        assert_eq!(aggregation_expr_map.len(), 5);
    }
}
