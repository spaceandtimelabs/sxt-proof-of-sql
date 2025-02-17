use super::*;
use proof_of_sql_parser::intermediate_ast::AliasedResultExpr;
use sqlparser::ast::Ident;

#[must_use]
/// Producing a postprocessing object that represents a group by operation.
pub fn group_by_postprocessing(
    cols: &[&str],
    result_exprs: &[AliasedResultExpr],
) -> OwnedTablePostprocessing {
    let ids: Vec<Ident> = cols.iter().map(|col| (*col).into()).collect();
    OwnedTablePostprocessing::new_group_by(
        GroupByPostprocessing::try_new(ids, result_exprs.to_vec()).unwrap(),
    )
}

/// Producing a postprocessing object that represents a select operation.
/// # Panics
///
/// This function may panic if the internal structures cannot be created properly, although this is unlikely under normal circumstances.
#[must_use]
pub fn select_expr(result_exprs: &[AliasedResultExpr]) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_select(SelectPostprocessing::new(result_exprs.to_vec()))
}

/// Producing a postprocessing object that represents a slice operation.
#[must_use]
pub fn slice(limit: Option<u64>, offset: Option<i64>) -> OwnedTablePostprocessing {
    OwnedTablePostprocessing::new_slice(SlicePostprocessing::new(limit, offset))
}

/// Producing a postprocessing object that represents an order by operation.
#[must_use]
pub fn orders(indexes: &[usize], directions: &[bool]) -> OwnedTablePostprocessing {
    let index_direction_pairs: Vec<(usize, bool)> = indexes
        .iter()
        .copied()
        .zip(directions.iter().copied())
        .collect();
    OwnedTablePostprocessing::new_order_by(OrderByPostprocessing::new(index_direction_pairs))
}
