//! This module contains new lightweight postprocessing for non-provable components.
mod error;
pub use error::{PostprocessingError, PostprocessingResult};

mod evaluate;
pub use evaluate::PostprocessingEvaluator;
#[cfg(test)]
mod evaluate_test;

mod owned_table_postprocessing;

mod postprocessing_step;
pub use owned_table_postprocessing::{apply_postprocessing_steps, OwnedTablePostprocessing};
pub use postprocessing_step::PostprocessingStep;
#[cfg(test)]
pub mod test_utility;

mod order_by_expr;
pub use order_by_expr::OrderByExpr;
#[cfg(test)]
mod order_by_expr_test;

mod slice_expr;
pub use slice_expr::SliceExpr;
#[cfg(test)]
mod slice_expr_test;
