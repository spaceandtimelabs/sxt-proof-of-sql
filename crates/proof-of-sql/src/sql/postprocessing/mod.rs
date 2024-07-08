//! This module contains new lightweight postprocessing for non-provable components.
mod error;
#[allow(unused_imports)]
pub use error::{PostprocessingError, PostprocessingResult};
mod owned_table_postprocessing;

mod postprocessing_step;
#[allow(unused_imports)]
pub use owned_table_postprocessing::{apply_postprocessing_steps, OwnedTablePostprocessing};
pub use postprocessing_step::PostprocessingStep;
#[cfg(test)]
pub mod test_utility;

mod slice_expr;
pub use slice_expr::SliceExpr;

#[cfg(test)]
mod slice_expr_test;
