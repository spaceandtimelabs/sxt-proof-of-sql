use super::*;
use proof_of_sql_parser::intermediate_ast::{AliasedResultExpr, OrderBy, OrderByDirection};

pub fn select_expr(result_exprs: &[AliasedResultExpr]) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_select(SelectPostprocessing::new(result_exprs.to_vec()))
}

pub fn slice(limit: Option<u64>, offset: Option<i64>) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_slice(SlicePostprocessing::new(limit, offset))
}

pub fn orders(cols: &[&str], directions: &[OrderByDirection]) -> OwnedTablePostprocessing {
    let by_exprs = cols
        .iter()
        .zip(directions.iter())
        .map(|(col, direction)| OrderBy {
            expr: col.parse().unwrap(),
            direction: *direction,
        })
        .collect();
    OwnedTablePostprocessing::new_order_by(OrderByPostprocessing::new(by_exprs))
}
