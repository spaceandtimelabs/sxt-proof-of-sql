use super::PostprocessingResult;
use core::fmt::Debug;
use proof_of_sql::base::{database::OwnedTable, scalar::Scalar};

/// A trait for postprocessing steps that can be applied to an `OwnedTable`.
pub trait PostprocessingStep<S: Scalar>: Debug + Send + Sync {
    /// Apply the postprocessing step to the `OwnedTable` and return the result.
    fn apply(&self, owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>>;
}
