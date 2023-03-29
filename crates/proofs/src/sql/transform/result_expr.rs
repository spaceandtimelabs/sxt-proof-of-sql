use crate::sql::proof::TransformExpr;

use dyn_partial_eq::DynPartialEq;

/// The result expression is used to transform the results of a query
#[derive(Default, Debug, DynPartialEq, PartialEq)]
pub struct ResultExpr;

impl TransformExpr for ResultExpr {
    // todo(joe): fill this implementation with the batch transformations
}
