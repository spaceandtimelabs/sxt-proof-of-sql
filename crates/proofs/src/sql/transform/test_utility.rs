use super::{OrderByExprs, ResultExpr, SliceExpr};
use crate::sql::transform::CompositionExpr;
use crate::sql::transform::DataFrameExpr;

use proofs_sql::intermediate_ast::{OrderBy, OrderByDirection};

pub fn result() -> Box<ResultExpr> {
    Box::default()
}

pub fn composite_result(tranformations: Box<dyn DataFrameExpr>) -> Box<ResultExpr> {
    Box::new(ResultExpr::new(Box::new(CompositionExpr::new(
        tranformations,
    ))))
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
