use crate::base::{database::OwnedTable, scalar::Scalar};
use std::fmt::Debug;

/// A trait for nodes that can apply transformations to a `OwnedTable`.
pub trait OwnedTablePostprocessing<S: Scalar>: Debug + Send + Sync {
    /// Apply the transformation to the `OwnedTable` and return the result.
    #[allow(dead_code)]
    fn apply_transformation(&self, owned_table: OwnedTable<S>) -> Option<OwnedTable<S>>;
}
