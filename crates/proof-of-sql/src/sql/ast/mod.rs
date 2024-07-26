//! This module proves provable ASTs.
mod aliased_provable_expr_plan;
pub(crate) use aliased_provable_expr_plan::AliasedProvableExprPlan;

mod filter_result_expr;
pub(crate) use filter_result_expr::FilterResultExpr;

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

mod filter_expr;
pub(crate) use filter_expr::FilterExpr;
#[cfg(test)]
pub(crate) use filter_expr::OstensibleFilterExpr;
#[cfg(all(test, feature = "blitzar"))]
mod filter_expr_test;
#[cfg(all(test, feature = "blitzar"))]
mod filter_expr_test_dishonest_prover;

mod bitwise_verification;
use bitwise_verification::*;
#[cfg(test)]
mod bitwise_verification_test;

mod provable_expr_plan;
pub(crate) use provable_expr_plan::ProvableExprPlan;

mod provable_expr;
pub(crate) use provable_expr::ProvableExpr;
#[cfg(all(test, feature = "blitzar"))]
mod provable_expr_test;

mod projection_expr;
pub(crate) use projection_expr::ProjectionExpr;
#[cfg(all(test, feature = "blitzar"))]
mod projection_expr_test;

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
    try_add_subtract_column_types, try_multiply_column_types,
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

mod dense_filter_expr;
pub(crate) use dense_filter_expr::DenseFilterExpr;
#[cfg(test)]
pub(crate) use dense_filter_expr::OstensibleDenseFilterExpr;
#[cfg(all(test, feature = "blitzar"))]
mod dense_filter_expr_test;
#[cfg(all(test, feature = "blitzar"))]
mod dense_filter_expr_test_dishonest_prover;

mod dense_filter_util;
pub(crate) use dense_filter_util::{
    filter_column_by_index, filter_columns, fold_columns, fold_vals,
};
#[cfg(test)]
mod dense_filter_util_test;

mod group_by_expr;
pub(crate) use group_by_expr::GroupByExpr;

#[cfg(all(test, feature = "blitzar"))]
mod group_by_expr_test;

mod group_by_util;
use group_by_util::aggregate_columns;
#[cfg(test)]
mod group_by_util_test;

mod proof_plan;
pub use proof_plan::ProofPlan;

#[cfg(test)]
mod demo_test_expr;
