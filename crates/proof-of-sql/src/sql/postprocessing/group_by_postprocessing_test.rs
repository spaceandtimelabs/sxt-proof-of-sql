use crate::sql::postprocessing::{group_by_postprocessing::*, PostprocessingError};
use indexmap::indexmap;
use proof_of_sql_parser::{intermediate_ast::AggregationOperator, utility::*};

#[test]
fn we_cannot_have_invalid_group_bys() {
    // Column in result but not in group by or aggregation
    let expr = add(sum(col("a")), col("b")); // b is not in group by or aggregation
    let res = GroupByPostprocessing::try_new(vec![ident("a")], vec![aliased_expr(expr, "res")]);
    assert!(matches!(
        res,
        Err(PostprocessingError::IdentifierNotInAggregationOperatorOrGroupByClause(_))
    ));

    // Nested aggregation
    let expr = sum(max(col("a"))); // Nested aggregation
    let res = GroupByPostprocessing::try_new(vec![ident("a")], vec![aliased_expr(expr, "res")]);
    assert!(matches!(
        res,
        Err(PostprocessingError::NestedAggregationInGroupByClause(_))
    ));
}

#[test]
fn we_can_make_group_by_postprocessing() {
    // SELECT SUM(a) + 2 as c0, SUM(b + a) as c1 FROM tab GROUP BY a, b
    let res = GroupByPostprocessing::try_new(
        vec![ident("a"), ident("b")],
        vec![
            aliased_expr(add(sum(col("a")), lit(2)), "c0"),
            aliased_expr(sum(add(col("b"), col("a"))), "c1"),
        ],
    )
    .unwrap();
    assert_eq!(res.group_by(), &[ident("a"), ident("b")]);
    assert_eq!(
        res.remainder_exprs(),
        &[
            aliased_expr(add(col("__col_agg_0"), lit(2)), "c0"),
            aliased_expr(col("__col_agg_1"), "c1"),
        ]
    );
    assert_eq!(
        res.aggregation_expr_map(),
        &indexmap! {
            (AggregationOperator::Sum, *col("a")) => ident("__col_agg_0"),
            (AggregationOperator::Sum, *add(col("b"), col("a"))) => ident("__col_agg_1"),
        }
    );
}
