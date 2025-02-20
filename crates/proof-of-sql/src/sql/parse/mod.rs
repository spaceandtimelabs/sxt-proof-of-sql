//! This module contains conversion of intermediate AST to provable AST and a non-provable component if necessary.
mod error;

pub use error::ConversionError;
pub(crate) use error::ConversionResult;

mod enriched_expr;
pub(crate) use enriched_expr::EnrichedExpr;

#[cfg(all(test, feature = "blitzar"))]
mod query_expr_tests;

mod query_expr;
pub use query_expr::QueryExpr;

mod filter_exec_builder;
pub(crate) use filter_exec_builder::FilterExecBuilder;

/// TODO: add docs
pub(crate) mod query_context;
pub(crate) use query_context::QueryContext;

mod query_context_builder;
pub(crate) use query_context_builder::{QueryContextBuilder, type_check_binary_operation};

mod dyn_proof_expr_builder;
pub(crate) use dyn_proof_expr_builder::DynProofExprBuilder;

mod where_expr_builder;
pub(crate) use where_expr_builder::WhereExprBuilder;
#[cfg(test)]
mod where_expr_builder_tests;
