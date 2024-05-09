//! TODO: add docs
mod filter_result_expr;
pub(crate) use filter_result_expr::FilterResultExpr;

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

mod literal_expr;
use literal_expr::LiteralExpr;
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

#[cfg(all(test, feature = "blitzar"))]
mod test_expr;

#[cfg(test)]
pub(crate) mod test_utility;

mod column_expr;
pub(crate) use column_expr::ColumnExpr;

mod dense_filter_expr;
pub(crate) use dense_filter_expr::DenseFilterExpr;
#[cfg(test)]
pub(crate) use dense_filter_expr::OstensibleDenseFilterExpr;
#[cfg(all(test, feature = "blitzar"))]
mod dense_filter_expr_test;
#[cfg(all(test, feature = "blitzar"))]
mod dense_filter_expr_test_dishonest_prover;

mod dense_filter_util;
#[cfg(test)]
pub(crate) use dense_filter_util::fold_vals;
pub(crate) use dense_filter_util::{filter_column_by_index, filter_columns};
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
