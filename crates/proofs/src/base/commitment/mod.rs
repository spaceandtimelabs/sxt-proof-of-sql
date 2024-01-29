//! Types for creation and utilization of cryptographic commitments to proof-of-sql data.
use crate::base::scalar::Scalar;
pub use blitzar::compute::{init_backend, init_backend_with_config, BackendConfig};
use core::ops::AddAssign;
use curve25519_dalek::ristretto::RistrettoPoint;

mod committable_column;
pub use committable_column::CommittableColumn;

mod vec_commitment_ext;
pub use vec_commitment_ext::VecCommitmentExt;

mod column_bounds;
use super::scalar::ArkScalar;
pub use column_bounds::{ColumnBounds, ColumnBoundsMismatch};

mod column_commitment_metadata;
pub use column_commitment_metadata::{ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch};

mod column_commitment_metadata_map;
pub use column_commitment_metadata_map::{
    ColumnCommitmentMetadataMap, ColumnCommitmentMetadataMapExt, ColumnCommitmentsMismatch,
};

mod column_commitments;
pub use column_commitments::{
    AppendColumnCommitmentsError, ColumnCommitments, DuplicateIdentifiers,
};

/// A trait for using commitment schemes generically.
pub trait Commitment:
    AddAssign
    + Sized
    + Default
    + Copy
    + core::ops::Neg<Output = Self>
    + Eq
    + core::ops::Sub<Output = Self>
    + core::fmt::Debug
    + std::marker::Sync
    + std::marker::Send
{
    /// The associated scalar that the commitment is for.
    /// There are multiple possible commitment schemes for a scalar, but only one scalar for any commitment.
    type Scalar: Scalar
        + for<'a> core::ops::Mul<&'a Self, Output = Self>
        + core::ops::Mul<Self, Output = Self>
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>;
    /// Computes the commitment to the vector of scalars that are 1 in the given range and 0 elsewhere.
    fn compute_ones_commit(range: core::ops::Range<u64>) -> Self;
}

impl Commitment for RistrettoPoint {
    type Scalar = ArkScalar;

    fn compute_ones_commit(range: core::ops::Range<u64>) -> Self {
        blitzar::compute::get_one_curve25519_commit(range.end)
            - blitzar::compute::get_one_curve25519_commit(range.start)
    }
}
