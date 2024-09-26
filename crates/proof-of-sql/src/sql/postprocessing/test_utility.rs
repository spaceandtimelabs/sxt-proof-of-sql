use super::*;
use proof_of_sql_parser::{
    intermediate_ast::{AliasedResultExpr, OrderBy, OrderByDirection},
    utility::ident,
    Identifier,
};

pub fn group_by_postprocessing(
    cols: &[&str],
    result_exprs: &[AliasedResultExpr],
) -> OwnedTablePostprocessing {
    let ids: Vec<Identifier> = cols.iter().map(|col| ident(col)).collect();
    OwnedTablePostprocessing::new_group_by(
        //TODO: add panic docs
        GroupByPostprocessing::try_new(ids, result_exprs.to_vec()).unwrap(),
    )
}

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
            //TODO: add panic docs
            expr: col.parse().unwrap(),
            direction: *direction,
        })
        .collect();
    OwnedTablePostprocessing::new_order_by(OrderByPostprocessing::new(by_exprs))
}
