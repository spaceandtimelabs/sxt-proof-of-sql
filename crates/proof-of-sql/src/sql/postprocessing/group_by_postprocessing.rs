use super::{PostprocessingError, PostprocessingResult, PostprocessingStep};
use crate::base::{
    database::{group_by_util::aggregate_columns, Column, OwnedColumn, OwnedTable},
    map::{indexmap, IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};
use bumpalo::Bump;
use itertools::{izip, Itertools};
use proof_of_sql_parser::intermediate_ast::AliasedResultExpr;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{Expr, Ident};

/// A group by expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupByPostprocessing {
    /// A list of `AliasedResultExpr` that exclusively use identifiers in the group by clause or results of aggregation expressions
    remainder_exprs: Vec<AliasedResultExpr>,

    /// A list of identifiers in the group by clause
    group_by_identifiers: Vec<Ident>,

    /// A list of aggregation expressions
    aggregation_exprs: Vec<(String, Expr, Ident)>,
}

/// Check whether multiple layers of aggregation exist within the same GROUP BY clause
/// since this is not allowed in SQL
///
/// If the context is within an aggregation function, then any aggregation function is considered nested.
/// Otherwise we need two layers of aggregation functions to be nested.
fn contains_nested_aggregation(expr: &Expr, is_agg: bool) -> bool {
    match expr {
        Expr::Identifier(_) | Expr::Value(_) | Expr::Wildcard => false,
        Expr::Function(function) => {
            is_agg
                || function.args.iter().any(|arg| match &arg {
                    sqlparser::ast::FunctionArg::Unnamed(
                        sqlparser::ast::FunctionArgExpr::Expr(arg_expr),
                    ) => contains_nested_aggregation(arg_expr, true),
                    sqlparser::ast::FunctionArg::Named { arg, .. } => {
                        if let sqlparser::ast::FunctionArgExpr::Expr(arg_expr) = arg {
                            contains_nested_aggregation(arg_expr, true)
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
        }
        Expr::BinaryOp { left, right, .. } => {
            contains_nested_aggregation(left, is_agg) || contains_nested_aggregation(right, is_agg)
        }
        Expr::UnaryOp { expr, .. } => contains_nested_aggregation(expr, is_agg),
        _ => false,
    }
}

/// Get identifiers NOT in aggregate functions
fn get_free_identifiers_from_expr(expr: &Expr) -> IndexSet<Ident> {
    match expr {
        Expr::Identifier(identifier) => IndexSet::from_iter([identifier.clone()]),
        Expr::Value(_) | Expr::Function(_) | Expr::Wildcard => IndexSet::default(),

        Expr::BinaryOp { left, right, .. } => {
            let mut left_identifiers = get_free_identifiers_from_expr(left);
            let right_identifiers = get_free_identifiers_from_expr(right);
            left_identifiers.extend(right_identifiers);
            left_identifiers
        }
        Expr::UnaryOp { expr, .. } => get_free_identifiers_from_expr(expr),
        _ => IndexSet::default(),
    }
}

/// Get aggregate expressions from an expression as well as the remainder
///
/// The idea here is to recursively traverse the expression tree and collect all the aggregation expressions
/// and then label them as new columns post-aggregation and replace them with these new columns so that
/// the post-aggregation expression tree doesn't contain any aggregation expressions and can be simply evaluated.
/// # Panics
///
/// Will panic if the key for an aggregation expression cannot be parsed as a valid identifier
/// or if there are issues retrieving an identifier from the map.
fn get_aggregate_and_remainder_expressions(
    expr: Expr,
    aggregation_expr_map: &mut IndexMap<(String, Expr), Ident>,
) -> Result<Expr, PostprocessingError> {
    match expr {
        Expr::Identifier(_) | Expr::Value(_) | Expr::Wildcard => Ok(expr),
        Expr::Function(function) => {
            let key = (function.name.to_string(), Expr::Function(function.clone()));

            if let Some(ident) = aggregation_expr_map.get(&key) {
                Ok(Expr::Identifier(ident.clone()))
            } else {
                let new_ident = Ident {
                    value: format!("__col_agg_{}", aggregation_expr_map.len()),
                    quote_style: None,
                };

                aggregation_expr_map.insert(key, new_ident.clone());
                Ok(Expr::Identifier(new_ident))
            }
        }
        Expr::BinaryOp { op, left, right } => {
            let left_remainder =
                get_aggregate_and_remainder_expressions(*left, aggregation_expr_map)?;
            let right_remainder =
                get_aggregate_and_remainder_expressions(*right, aggregation_expr_map)?;
            Ok(Expr::BinaryOp {
                op,
                left: Box::new(left_remainder),
                right: Box::new(right_remainder),
            })
        }

        Expr::UnaryOp { op, expr } => {
            let remainder = get_aggregate_and_remainder_expressions(*expr, aggregation_expr_map)?;
            Ok(Expr::UnaryOp {
                op,
                expr: Box::new(remainder),
            })
        }
        _ => Ok(expr),
    }
}

/// Given an `AliasedResultExpr`, check if it is legitimate and if so grab the relevant aggregation expression
/// # Panics
///
/// Will panic if there is an issue retrieving the first element from the difference of free identifiers and group-by identifiers, indicating a logical inconsistency in the identifiers.
fn check_and_get_aggregation_and_remainder(
    expr: AliasedResultExpr,
    group_by_identifiers: &[Ident],
    aggregation_expr_map: &mut IndexMap<(String, Expr), Ident>,
) -> PostprocessingResult<AliasedResultExpr> {
    let converted_expr: Expr = (*expr.expr).clone().into();
    let free_identifiers = get_free_identifiers_from_expr(&converted_expr);
    let group_by_identifier_set = group_by_identifiers
        .iter()
        .cloned()
        .collect::<IndexSet<_>>();
    if contains_nested_aggregation(&converted_expr, false) {
        return Err(PostprocessingError::NestedAggregationInGroupByClause {
            error: format!("Nested aggregations found {:?}", expr.expr),
        });
    }
    if free_identifiers.is_subset(&group_by_identifier_set) {
        let remainder_expr =
            get_aggregate_and_remainder_expressions((*expr.expr).into(), aggregation_expr_map)?;
        Ok(AliasedResultExpr {
            alias: expr.alias,
            expr: Box::new(remainder_expr),
        })
    } else {
        let diff = free_identifiers
            .difference(&group_by_identifier_set)
            .next()
            .unwrap();
        Err(
            PostprocessingError::IdentNotInAggregationOperatorOrGroupByClause {
                column: diff.clone(),
            },
        )
    }
}

impl GroupByPostprocessing {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn try_new(
        by_ids: Vec<Ident>,
        aliased_exprs: Vec<AliasedResultExpr>,
    ) -> PostprocessingResult<Self> {
        let mut aggregation_expr_map: IndexMap<(String, Expr), Ident> = IndexMap::default();
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
    #[must_use]
    pub fn group_by(&self) -> &[Ident] {
        &self.group_by_identifiers
    }

    /// Get remainder expressions for SELECT
    #[must_use]
    pub fn remainder_exprs(&self) -> &[AliasedResultExpr] {
        &self.remainder_exprs
    }

    /// Get aggregation expressions
    #[must_use]
    pub fn aggregation_exprs(&self) -> &[(String, Expr, Ident)] {
        &self.aggregation_exprs
    }
}

impl<S: Scalar> PostprocessingStep<S> for GroupByPostprocessing {
    /// Apply the group by transformation to the given `OwnedTable`.
    #[allow(clippy::too_many_lines)]
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        // First evaluate all the aggregated columns
        let alloc = Bump::new();
        let evaluated_columns = self
            .aggregation_exprs
            .iter()
            .map(|(agg_op, expr, id)| -> PostprocessingResult<_> {
                let evaluated_owned_column = owned_table.evaluate(expr)?;
                Ok((agg_op.to_string(), (id.clone(), evaluated_owned_column)))
            })
            .process_results(|iter| {
                iter.fold(
                    IndexMap::<String, Vec<_>>::default(),
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

        let (sum_identifiers, sum_columns): (Vec<_>, Vec<_>) =
            evaluated_columns
                .get("Sum")
                .map_or((vec![], vec![]), |tuple| {
                    tuple
                        .iter()
                        .map(|(id, c)| (id.clone(), Column::<S>::from_owned_column(c, &alloc)))
                        .unzip()
                });

        let (max_identifiers, max_columns): (Vec<_>, Vec<_>) =
            evaluated_columns
                .get("Max")
                .map_or((vec![], vec![]), |tuple| {
                    tuple
                        .iter()
                        .map(|(id, c)| (id.clone(), Column::<S>::from_owned_column(c, &alloc)))
                        .unzip()
                });

        let (min_identifiers, min_columns): (Vec<_>, Vec<_>) =
            evaluated_columns
                .get("Min")
                .map_or((vec![], vec![]), |tuple| {
                    tuple
                        .iter()
                        .map(|(id, c)| (id.clone(), Column::<S>::from_owned_column(c, &alloc)))
                        .unzip()
                });

        let aggregation_results = aggregate_columns(
            &alloc,
            &group_by_ins,
            &sum_columns,
            &max_columns,
            &min_columns,
            &selection_in,
        )?;

        // Finally do another round of evaluation to get the final result
        // Gather the results into a new OwnedTable
        let group_by_outs = aggregation_results
            .group_by_columns
            .iter()
            .zip(self.group_by_identifiers.iter())
            .map(|(column, id)| Ok((id.clone(), OwnedColumn::from(column))));

        let sum_outs = izip!(
            aggregation_results.sum_columns,
            sum_identifiers,
            sum_columns,
        )
        .map(|(c_out, id, c_in)| {
            Ok((
                id,
                OwnedColumn::try_from_scalars(c_out, c_in.column_type())?,
            ))
        });

        let max_outs = izip!(
            aggregation_results.max_columns,
            max_identifiers,
            max_columns,
        )
        .map(|(c_out, id, c_in)| {
            Ok((
                id,
                OwnedColumn::try_from_option_scalars(c_out, c_in.column_type())?,
            ))
        });

        let min_outs = izip!(
            aggregation_results.min_columns,
            min_identifiers,
            min_columns,
        )
        .map(|(c_out, id, c_in)| {
            Ok((
                id,
                OwnedColumn::try_from_option_scalars(c_out, c_in.column_type())?,
            ))
        });

        //TODO: When we have NULLs we need to differentiate between count(1) and count(expression)
        let count_column = OwnedColumn::BigInt(aggregation_results.count_column.to_vec());
        let count_outs =
            evaluated_columns.get("Count").into_iter().flatten().map(
                |(id, _)| -> PostprocessingResult<_> { Ok((id.clone(), count_column.clone())) },
            );

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
            OwnedTable::try_new(indexmap! {"__count__".into() => count_column})?
        } else {
            new_owned_table
        };

        let result = self
            .remainder_exprs
            .iter()
            .map(|aliased_expr| -> PostprocessingResult<_> {
                let expr_as_expr: Expr = (*aliased_expr.expr).clone().into();
                let column = target_table.evaluate(&expr_as_expr)?;
                let alias: Ident = aliased_expr.alias.into();
                Ok((alias, column))
            })
            .process_results(|iter| OwnedTable::try_from_iter(iter))??;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proof_of_sql_parser::utility::*;

    #[test]
    fn we_can_detect_nested_aggregation() {
        // SUM(SUM(a))
        let expr = (*sum(sum(col("a")))).into();
        assert!(contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // MAX(a) + SUM(b)
        let expr = (*add(max(col("a")), sum(col("b")))).into();
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + SUM(b)
        let expr = (*add(col("a"), sum(col("b")))).into();
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // SUM(a) + b - SUM(2 * c)
        let expr = (*sub(add(sum(col("a")), col("b")), sum(mul(lit(2), col("c"))))).into();
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + COUNT(SUM(a))
        let expr = (*add(col("a"), count(sum(col("a"))))).into();
        assert!(contains_nested_aggregation(&expr, false));
        assert!(contains_nested_aggregation(&expr, true));

        // a + b + 1
        let expr = (*add(add(col("a"), col("b")), lit(1))).into();
        assert!(!contains_nested_aggregation(&expr, false));
        assert!(!contains_nested_aggregation(&expr, true));
    }

    #[test]
    fn we_can_get_free_identifiers_from_expr() {
        // Literal
        let expr = (*lit("Not an identifier")).into();
        let expected: IndexSet<Ident> = IndexSet::default();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // a + b + 1
        let expr = (*add(add(col("a"), col("b")), lit(1))).into();
        let expected: IndexSet<Ident> = ["a".into(), "b".into()].into_iter().collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // ! (a == b || c >= a)
        let expr = (*not(or(equal(col("a"), col("b")), ge(col("c"), col("a"))))).into();
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into()].into_iter().collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // SUM(a + b) * 2
        let expr = (*mul(sum(add(col("a"), col("b"))).into(), lit(2))).into();
        let expected: IndexSet<Ident> = IndexSet::default();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);

        // (COUNT(a + b) + c) * d
        let expr = (*mul(
            add(count(add(col("a"), col("b"))).into(), col("c")),
            col("d"),
        ))
        .into();
        let expected: IndexSet<Ident> = ["c".into(), "d".into()].into_iter().collect();
        let actual = get_free_identifiers_from_expr(&expr);
        assert_eq!(actual, expected);
    }

    #[test]
    fn we_can_get_aggregate_and_remainder_expressions() {
        let mut aggregation_expr_map: IndexMap<(String, Expr), Ident> = IndexMap::default();

        // SUM(a) + b
        let expr = add(sum(col("a")), col("b"));
        let remainder_expr =
            get_aggregate_and_remainder_expressions((*expr).into(), &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&("SUM".to_string(), (*col("a")).into())],
            "__col_agg_0".into()
        );
        let expected_remainder: Expr = (*add(col("__col_agg_0"), col("b"))).into();
        assert_eq!(remainder_expr, Ok(expected_remainder));
        assert_eq!(aggregation_expr_map.len(), 1);

        // SUM(a) + SUM(b)
        let expr = add(sum(col("a")), sum(col("b")));
        let remainder_expr =
            get_aggregate_and_remainder_expressions((*expr).into(), &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&("SUM".to_string(), (*col("a")).into())],
            "__col_agg_0".into()
        );
        assert_eq!(
            aggregation_expr_map[&("SUM".to_string(), (*col("b")).into())],
            "__col_agg_1".into()
        );
        let expected_remainder: Expr = (*add(col("__col_agg_0"), col("__col_agg_1"))).into();
        assert_eq!(remainder_expr, Ok(expected_remainder));

        // MAX(a + 1) + MIN(2 * b - 4) + c
        let expr = add(
            add(
                max(add(col("a"), lit(1))),
                min(sub(mul(lit(2), col("b")), lit(4))),
            ),
            col("c"),
        );
        let remainder_expr =
            get_aggregate_and_remainder_expressions((*expr).into(), &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&("MAX".to_string(), (*add(col("a"), lit(1))).into())],
            "__col_agg_2".into()
        );
        assert_eq!(
            aggregation_expr_map[&(
                "MIN".to_string(),
                (*sub(mul(lit(2), col("b")), lit(4))).into()
            )],
            "__col_agg_3".into()
        );
        let expected_remainder: Expr =
            (*add(add(col("__col_agg_2"), col("__col_agg_3")), col("c"))).into();
        assert_eq!(remainder_expr, Ok(expected_remainder));

        // COUNT(2 * a) * 2 + SUM(b) + 1
        let expr = add(
            add(mul(count(mul(lit(2), col("a"))), lit(2)), sum(col("b"))),
            lit(1),
        );
        let remainder_expr =
            get_aggregate_and_remainder_expressions((*expr).into(), &mut aggregation_expr_map);
        assert_eq!(
            aggregation_expr_map[&("COUNT".to_string(), (*mul(lit(2), col("a"))).into())],
            "__col_agg_4".into()
        );
        let expected_remainder: Expr = (*add(
            add(mul(col("__col_agg_4"), lit(2)), col("__col_agg_1")),
            lit(1),
        ))
        .into();
        assert_eq!(remainder_expr, Ok(expected_remainder));
    }
}
