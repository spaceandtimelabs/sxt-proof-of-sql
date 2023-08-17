use super::{GroupByExpr, OrderByExprs, ResultExpr, SelectExpr, SliceExpr};
use crate::sql::transform::CompositionExpr;
use crate::sql::transform::DataFrameExpr;

use proofs_sql::intermediate_ast::ResultColumn;
use proofs_sql::intermediate_ast::{
    AggExpr, AliasedResultExpr, Expression, OrderBy, OrderByDirection,
};

pub fn result(result_schema: &[(&str, &str)]) -> Box<ResultExpr> {
    Box::new(ResultExpr::new_with_result_schema(schema(result_schema)))
}

pub fn schema(columns: &[(&str, &str)]) -> Vec<ResultColumn> {
    columns
        .iter()
        .map(|(name, alias)| ResultColumn {
            name: name.parse().unwrap(),
            alias: alias.parse().unwrap(),
        })
        .collect()
}

pub fn composite_result(transformations: Vec<Box<dyn DataFrameExpr>>) -> Box<ResultExpr> {
    let mut composition = CompositionExpr::default();

    for transformation in transformations {
        composition.add(transformation);
    }

    Box::new(ResultExpr::new_with_transformation(Box::new(composition)))
}

pub fn orders(cols: &[&str], directions: &[OrderByDirection]) -> Box<dyn DataFrameExpr> {
    let by_exprs = cols
        .iter()
        .zip(directions.iter())
        .map(|(col, direction)| OrderBy {
            expr: col.parse().unwrap(),
            direction: direction.clone(),
        })
        .collect();

    Box::new(OrderByExprs::new(by_exprs))
}

pub fn slice(limit: u64, offset: i64) -> Box<dyn DataFrameExpr> {
    Box::new(SliceExpr::new(limit, offset))
}

pub fn agg_expr(agg_type: &str, name: &str, alias: &str) -> AliasedResultExpr {
    match agg_type {
        "max" => AliasedResultExpr {
            expr: proofs_sql::intermediate_ast::ResultExpr::Agg(AggExpr::Max(Box::new(
                Expression::Column(name.parse().unwrap()),
            ))),
            alias: alias.parse().unwrap(),
        },
        "min" => AliasedResultExpr {
            expr: proofs_sql::intermediate_ast::ResultExpr::Agg(AggExpr::Min(Box::new(
                Expression::Column(name.parse().unwrap()),
            ))),
            alias: alias.parse().unwrap(),
        },
        "sum" => AliasedResultExpr {
            expr: proofs_sql::intermediate_ast::ResultExpr::Agg(AggExpr::Sum(Box::new(
                Expression::Column(name.parse().unwrap()),
            ))),
            alias: alias.parse().unwrap(),
        },
        "count" => AliasedResultExpr {
            expr: proofs_sql::intermediate_ast::ResultExpr::Agg(AggExpr::Count(Box::new(
                Expression::Column(name.parse().unwrap()),
            ))),
            alias: alias.parse().unwrap(),
        },
        _ => panic!("Unsupported agg type"),
    }
}

pub fn groupby(
    by_exprs: Vec<(&str, Option<&str>)>,
    agg_exprs: Vec<AliasedResultExpr>,
) -> Box<dyn DataFrameExpr> {
    let by_exprs = by_exprs
        .iter()
        .map(|(name, alias)| {
            (
                name.parse().unwrap(),
                alias.as_ref().map(|alias| alias.parse().unwrap()),
            )
        })
        .collect::<Vec<_>>();

    Box::new(GroupByExpr::new(by_exprs, agg_exprs))
}

pub fn select(result_schema: &[(&str, &str)]) -> Box<dyn DataFrameExpr> {
    Box::new(SelectExpr::new(schema(result_schema)))
}
