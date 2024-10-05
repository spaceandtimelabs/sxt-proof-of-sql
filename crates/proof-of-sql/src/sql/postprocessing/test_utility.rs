use super::*;
use proof_of_sql_parser::{
    intermediate_ast::{AliasedResultExpr, OrderBy, OrderByDirection},
    utility::ident,
    Identifier,
};

#[must_use]
pub fn group_by_postprocessing(
    cols: &[&str],
    result_exprs: &[AliasedResultExpr],
) -> OwnedTablePostprocessing {
    let ids: Vec<Identifier> = cols.iter().map(|col| ident(col)).collect();
    OwnedTablePostprocessing::new_group_by(
        GroupByPostprocessing::try_new(ids, result_exprs.to_vec()).unwrap(),
    )
}

///
/// # Panics
///
/// This function may panic if the internal structures cannot be created properly, although this is unlikely under normal circumstances.
#[must_use]
pub fn select_expr(result_exprs: &[AliasedResultExpr]) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_select(SelectPostprocessing::new(result_exprs.to_vec()))
}

#[must_use]
pub fn slice(limit: Option<u64>, offset: Option<i64>) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_slice(SlicePostprocessing::new(limit, offset))
}

#[must_use]
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
