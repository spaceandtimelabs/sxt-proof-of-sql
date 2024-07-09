use super::PostprocessingResult;
use crate::base::{database::OwnedTable, scalar::Scalar};

/// An enum for nodes that can apply postprocessing to a `OwnedTable`.
#[derive(Debug, Clone)]
pub enum OwnedTablePostprocessing<S: Scalar> {
    #[allow(dead_code)]
    Placeholder(std::marker::PhantomData<S>),
}

impl<S: Scalar> OwnedTablePostprocessing<S> {
    /// Apply the postprocessing step to the `OwnedTable` and return the result.
    #[allow(dead_code)]
    fn apply(&self, _owned_table: OwnedTable<S>) -> PostprocessingResult<OwnedTable<S>> {
        todo!()
    }
}
