//! This module contains new lightweight postprocessing for non-provable components.
mod error;
#[allow(unused_imports)]
pub(crate) use error::{PostprocessingError, PostprocessingResult};
mod owned_table_postprocessing;
#[allow(unused_imports)]
pub(crate) use owned_table_postprocessing::OwnedTablePostprocessing;
