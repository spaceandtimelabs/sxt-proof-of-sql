use super::{PostprocessingError, PostprocessingResult};
use indexmap::{IndexMap, IndexSet};
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

    /// An `IndexMap` of aggregation expressions
    aggregation_expr_map: IndexMap<(AggregationOperator, Expression), Identifier>,
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
        Expression::Column(identifier) => IndexSet::from([*identifier]),
        Expression::Literal(_) | Expression::Aggregation { .. } | Expression::Wildcard => {
            IndexSet::new()
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
        Expression::Column(_) | Expression::Literal(_) | Expression::Wildcard => expr.clone(),
        Expression::Aggregation { op, expr } => {
            let key = (op, (*expr).clone());
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
        return Err(PostprocessingError::NestedAggregationInGroupByClause(
            format!("Nested aggregations found {:?}", expr.expr),
        ));
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
        Err(PostprocessingError::IdentifierNotInAggregationOperatorOrGroupByClause(*diff))
    }
}

impl GroupByPostprocessing {
    /// Create a new group by expression containing the group by and aggregation expressions
    pub fn try_new(
        by_ids: Vec<Identifier>,
        aliased_exprs: Vec<AliasedResultExpr>,
    ) -> PostprocessingResult<Self> {
        let mut aggregation_expr_map: IndexMap<(AggregationOperator, Expression), Identifier> =
            IndexMap::new();
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
        Ok(Self {
            remainder_exprs,
            group_by_identifiers: by_ids,
            aggregation_expr_map,
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

    /// Get aggregation expression map
    pub fn aggregation_expr_map(&self) -> &IndexMap<(AggregationOperator, Expression), Identifier> {
        &self.aggregation_expr_map
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
        let expected: IndexSet<Identifier> = IndexSet::new();
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
        let expected: IndexSet<Identifier> = IndexSet::new();
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
            IndexMap::new();
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
