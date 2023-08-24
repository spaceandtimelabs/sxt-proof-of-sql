use super::{GroupByExpr, OrderByExprs, ResultExpr, SelectExpr, SliceExpr};
use crate::sql::proof::TransformExpr;
use crate::sql::transform::CompositionExpr;
use crate::sql::transform::DataFrameExpr;

use proofs_sql::intermediate_ast::{OrderBy, OrderByDirection};

use polars::prelude::{col, Expr};

pub fn select(result_schema: &[Expr]) -> Box<dyn DataFrameExpr> {
    Box::new(SelectExpr::new(result_schema.to_vec()))
}

pub fn schema(columns: &[(&str, &str)]) -> Vec<Expr> {
    columns
        .iter()
        .map(|(name, alias)| col(name).alias(alias))
        .collect()
}

pub fn result(columns: &[(&str, &str)]) -> Box<dyn TransformExpr> {
    let mut composition = CompositionExpr::default();
    composition.add(Box::new(SelectExpr::new(schema(columns))));
    Box::new(ResultExpr::new(Box::new(composition)))
}

pub fn slice(limit: u64, offset: i64) -> Box<dyn DataFrameExpr> {
    Box::new(SliceExpr::new(limit, offset))
}

pub fn composite_result(transformations: Vec<Box<dyn DataFrameExpr>>) -> Box<ResultExpr> {
    let mut composition = CompositionExpr::default();

    for transformation in transformations {
        composition.add(transformation);
    }

    Box::new(ResultExpr::new(Box::new(composition)))
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

pub fn agg_expr(agg_type: &str, name: &str, alias: &str) -> Expr {
    match agg_type {
        "max" => col(name).max().alias(alias),
        "min" => col(name).min().alias(alias),
        "sum" => col(name).sum().alias(alias),
        "count" => col(name).count().alias(alias),
        _ => panic!("Unsupported agg type"),
    }
}

pub fn groupby(by_exprs: Vec<Expr>, agg_exprs: Vec<Expr>) -> Box<dyn DataFrameExpr> {
    Box::new(GroupByExpr::new(by_exprs, agg_exprs))
}
