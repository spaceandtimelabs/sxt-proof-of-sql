//! This module contains new lightweight postprocessing for non-provable components.
mod error;
pub use error::{PostprocessingError, PostprocessingResult};

mod owned_table_postprocessing;

mod postprocessing_step;
pub use owned_table_postprocessing::{apply_postprocessing_steps, OwnedTablePostprocessing};
pub use postprocessing_step::PostprocessingStep;
#[cfg(test)]
/// Utility functions for testing postprocessing steps.
pub mod test_utility;

mod group_by_postprocessing;
pub use group_by_postprocessing::GroupByPostprocessing;
#[cfg(test)]
mod group_by_postprocessing_test;

mod order_by_postprocessing;
pub use order_by_postprocessing::OrderByPostprocessing;
#[cfg(test)]
mod order_by_postprocessing_test;

mod select_postprocessing;
pub use select_postprocessing::SelectPostprocessing;
#[cfg(test)]
mod select_postprocessing_test;

mod slice_postprocessing;
pub use slice_postprocessing::SlicePostprocessing;
#[cfg(test)]
mod slice_postprocessing_test;
