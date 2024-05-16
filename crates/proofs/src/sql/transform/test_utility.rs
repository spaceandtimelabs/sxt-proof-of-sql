use super::{
    record_batch_expr::RecordBatchExpr, GroupByExpr, OrderByExprs, ResultExpr, SelectExpr,
    SliceExpr,
};
use crate::{
    base::database::{INT128_PRECISION, INT128_SCALE},
    sql::transform::CompositionExpr,
};
use polars::prelude::{col, DataType, Expr, Literal, Series};
use proofs_sql::intermediate_ast::{OrderBy, OrderByDirection};

pub fn lit(value: i128) -> Expr {
    let literal = [value.to_string()].into_iter().collect::<Series>().lit();
    literal.cast(DataType::Decimal(
        Some(INT128_PRECISION),
        Some(INT128_SCALE),
    ))
}

pub fn select(result_schema: &[Expr]) -> Box<dyn RecordBatchExpr> {
    Box::new(SelectExpr::new(result_schema.to_vec()))
}

pub fn schema(columns: &[(&str, &str)]) -> Vec<Expr> {
    columns
        .iter()
        .map(|(name, alias)| col(name).alias(alias))
        .collect()
}

pub fn result(columns: &[(&str, &str)]) -> ResultExpr {
    let mut composition = CompositionExpr::default();
    composition.add(Box::new(SelectExpr::new(schema(columns))));
    ResultExpr::new(Box::new(composition))
}

pub fn slice(limit: u64, offset: i64) -> Box<dyn RecordBatchExpr> {
    Box::new(SliceExpr::new(limit, offset))
}

pub fn composite_result(transformations: Vec<Box<dyn RecordBatchExpr>>) -> ResultExpr {
    let mut composition = CompositionExpr::default();

    for transformation in transformations {
        composition.add(transformation);
    }

    ResultExpr::new(Box::new(composition))
}

pub fn orders(cols: &[&str], directions: &[OrderByDirection]) -> Box<dyn RecordBatchExpr> {
    let by_exprs = cols
        .iter()
        .zip(directions.iter())
        .map(|(col, direction)| OrderBy {
            expr: col.parse().unwrap(),
            direction: *direction,
        })
        .collect();

    Box::new(OrderByExprs::new(by_exprs))
}

pub fn groupby<T: IntoIterator<Item = Expr>, A: IntoIterator<Item = Expr>>(
    by_exprs: T,
    agg_exprs: A,
) -> Box<dyn RecordBatchExpr> {
    Box::new(GroupByExpr::new(
        by_exprs.into_iter().collect(),
        agg_exprs.into_iter().collect(),
    ))
}
