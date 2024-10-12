use super::PostprocessingResult;
use crate::base::{database::OwnedTable, scalar::Scalar};
use core::fmt::Debug;

/// A trait for postprocessing steps that can be applied to an `OwnedTable`.
pub trait PostprocessingStep<S: Scalar>: Debug + Send + Sync {
    /// Apply the postprocessing step to the `OwnedTable` and return the result.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>>;
}
