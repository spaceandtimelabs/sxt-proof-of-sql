//! This module proves provable expressions.
mod proof_expr;
pub use proof_expr::ProofExpr;
#[cfg(all(test, feature = "blitzar"))]
mod proof_expr_test;

mod aliased_dyn_proof_expr;
pub use aliased_dyn_proof_expr::AliasedDynProofExpr;

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

mod dyn_proof_expr;
pub use dyn_proof_expr::DynProofExpr;

mod literal_expr;
pub(crate) use literal_expr::LiteralExpr;
#[cfg(all(test, feature = "blitzar"))]
mod literal_expr_test;

mod and_expr;
pub(crate) use and_expr::AndExpr;
#[cfg(all(test, feature = "blitzar"))]
mod and_expr_test;

mod inequality_expr;
use inequality_expr::InequalityExpr;
#[cfg(all(test, feature = "blitzar"))]
mod inequality_expr_test;

mod or_expr;
use or_expr::OrExpr;
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
    add_subtract_columns, columns_to_scalar_slice, divide_columns, modulo_columns,
    multiply_columns, scale_and_add_subtract_eval,
};

mod equals_expr;
pub(crate) use equals_expr::EqualsExpr;
#[cfg(all(test, feature = "blitzar"))]
mod equals_expr_test;

mod table_expr;
pub use table_expr::TableExpr;

#[cfg(test)]
pub(crate) mod test_utility;

mod column_expr;
pub(crate) use column_expr::ColumnExpr;
#[cfg(all(test, feature = "blitzar"))]
mod column_expr_test;
