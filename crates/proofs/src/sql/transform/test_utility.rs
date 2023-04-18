use super::{GroupByExpr, OrderByExprs, ResultExpr, SelectExpr, SliceExpr};
use crate::sql::transform::CompositionExpr;
use crate::sql::transform::DataFrameExpr;

use proofs_sql::intermediate_ast::ResultColumn;
use proofs_sql::intermediate_ast::{AggExpr, OrderBy, OrderByDirection};
use proofs_sql::Identifier;

pub fn result() -> Box<ResultExpr> {
    let composition = CompositionExpr::default();

    Box::new(ResultExpr::new_with_transformation(Box::new(composition)))
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

pub fn groupby(
    by_exprs: Vec<(&str, Option<&str>)>,
    agg_exprs: Vec<AggExpr>,
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

pub fn select(columns: &[(&str, &str)]) -> Box<dyn DataFrameExpr> {
    let columns = columns
        .iter()
        .map(|(name, alias)| ResultColumn {
            name: name.parse::<Identifier>().unwrap(),
            alias: alias.parse::<Identifier>().unwrap(),
        })
        .collect::<Vec<_>>();

    Box::new(SelectExpr::new(columns))
}
