//! This module contains postprocessing for non-provable components.
mod result_expr;
pub use result_expr::ResultExpr;

#[cfg(test)]
pub mod test_utility;

mod composition_expr;
pub use composition_expr::CompositionExpr;

#[cfg(test)]
pub mod composition_expr_test;

mod data_frame_expr;
#[allow(deprecated)]
pub(crate) use data_frame_expr::DataFrameExpr;
mod record_batch_expr;
pub(crate) use record_batch_expr::impl_record_batch_expr_for_data_frame_expr;
pub use record_batch_expr::RecordBatchExpr;

mod order_by_exprs;
pub use order_by_exprs::OrderByExprs;

#[cfg(test)]
mod order_by_exprs_test;

#[cfg(test)]
pub(crate) use order_by_exprs::order_by_map_i128_to_utf8;

mod slice_expr;
pub use slice_expr::SliceExpr;

#[cfg(test)]
mod slice_expr_test;

mod select_expr;
pub use select_expr::SelectExpr;

#[cfg(test)]
mod select_expr_test;

mod group_by_expr;
#[cfg(test)]
pub(crate) use group_by_expr::group_by_map_i128_to_utf8;
pub use group_by_expr::GroupByExpr;

#[cfg(test)]
mod group_by_expr_test;

mod polars_conversions;
pub use polars_conversions::LiteralConversion;

mod polars_arithmetic;
pub use polars_arithmetic::SafeDivision;
mod to_polars_expr;
pub(crate) use to_polars_expr::ToPolarsExpr;
