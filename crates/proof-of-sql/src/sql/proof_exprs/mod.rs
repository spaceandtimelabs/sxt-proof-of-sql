//! This module proves provable expressions.
mod proof_expr;
pub(crate) use proof_expr::ProofExpr;
#[cfg(all(test, feature = "blitzar"))]
mod proof_expr_test;

mod aliased_dyn_proof_expr;
pub(crate) use aliased_dyn_proof_expr::AliasedDynProofExpr;

mod add_subtract_expr;
pub(crate) use add_subtract_expr::AddSubtractExpr;
#[cfg(all(test, feature = "blitzar"))]
mod add_subtract_expr_test;

mod aggregate_expr;
pub(crate) use aggregate_expr::AggregateExpr;

mod multiply_expr;
use multiply_expr::MultiplyExpr;
#[cfg(all(test, feature = "blitzar"))]
mod multiply_expr_test;

mod bitwise_verification;
use bitwise_verification::*;
#[cfg(test)]
mod bitwise_verification_test;

mod dyn_proof_expr;
pub(crate) use dyn_proof_expr::DynProofExpr;

mod literal_expr;
pub(crate) use literal_expr::LiteralExpr;
#[cfg(all(test, feature = "blitzar"))]
mod literal_expr_test;

mod and_expr;
use and_expr::AndExpr;
#[cfg(all(test, feature = "blitzar"))]
mod and_expr_test;

mod inequality_expr;
use inequality_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod inequality_expr_test;

mod or_expr;
use or_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod or_expr_test;

mod not_expr;
use not_expr::NotExpr;
#[cfg(all(test, feature = "blitzar"))]
mod not_expr_test;

mod comparison_util;
pub(crate) use comparison_util::scale_and_subtract;

mod numerical_util;
pub(crate) use numerical_util::{
    add_subtract_columns, multiply_columns, scale_and_add_subtract_eval,
};

mod equals_expr;
use equals_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod equals_expr_test;

mod sign_expr;
use sign_expr::*;
#[cfg(all(test, feature = "blitzar"))]
mod sign_expr_test;

mod table_expr;
pub(crate) use table_expr::TableExpr;

#[cfg(test)]
pub(crate) mod test_utility;

mod column_expr;
pub(crate) use column_expr::ColumnExpr;
#[cfg(all(test, feature = "blitzar"))]
mod column_expr_test;

#[allow(dead_code, unused_variables)]
mod range_check;

#[cfg(test)]
mod proof_expr_test_plan;
