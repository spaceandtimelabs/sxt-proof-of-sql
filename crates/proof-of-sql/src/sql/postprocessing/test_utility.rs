use super::*;
use crate::base::scalar::Scalar;
use proof_of_sql_parser::intermediate_ast::{OrderBy, OrderByDirection};

pub fn select(result_exprs: &[AliasedResultExpr]) -> OwnedTablePostprocessing<S>  {
    OwnedTablePostprocessing::<S>::new_select(SelectExpr::new(result_exprs.to_vec()))
}

pub fn slice<S: Scalar>(limit: Option<u64>, offset: Option<i64>) -> OwnedTablePostprocessing<S> {
    OwnedTablePostprocessing::<S>::new_slice(SliceExpr::new(limit, offset))
}

pub fn orders<S: Scalar>(
    cols: &[&str],
    directions: &[OrderByDirection],
) -> OwnedTablePostprocessing<S> {
    let by_exprs = cols
        .iter()
        .zip(directions.iter())
        .map(|(col, direction)| OrderBy {
            expr: col.parse().unwrap(),
            direction: *direction,
        })
        .collect();
    OwnedTablePostprocessing::<S>::new_order_by(OrderByExpr::new(by_exprs))
}
