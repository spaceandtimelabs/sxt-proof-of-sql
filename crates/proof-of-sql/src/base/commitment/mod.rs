//! Types for creation and utilization of cryptographic commitments to proof-of-sql data.
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
#[cfg(feature = "blitzar")]
pub use blitzar::{
    compute::{init_backend, init_backend_with_config, BackendConfig},
    proof::InnerProductProof,
};
use core::ops::{AddAssign, SubAssign};
mod committable_column;
pub use committable_column::CommittableColumn;

mod vec_commitment_ext;
pub use vec_commitment_ext::{NumColumnsMismatch, VecCommitmentExt};

mod column_bounds;
pub use column_bounds::{Bounds, ColumnBounds, NegativeBounds};

mod column_commitment_metadata;
pub use column_commitment_metadata::ColumnCommitmentMetadata;

mod column_commitment_metadata_map;
pub use column_commitment_metadata_map::{
    ColumnCommitmentMetadataMap, ColumnCommitmentMetadataMapExt, ColumnCommitmentsMismatch,
};

mod column_commitments;
pub use column_commitments::{AppendColumnCommitmentsError, ColumnCommitments, DuplicateIdents};

mod table_commitment;
pub use table_commitment::{
    AppendTableCommitmentError, MixedLengthColumns, NegativeRange, TableCommitment,
    TableCommitmentArithmeticError, TableCommitmentFromColumnsError,
};

mod query_commitments;
pub use query_commitments::{QueryCommitments, QueryCommitmentsExt};

/// Module for providing a mock commitment.
#[cfg(test)]
pub mod naive_commitment;

/// Module for providing a test commitment evaluation proof.
#[cfg(test)]
pub mod naive_evaluation_proof;

#[cfg(test)]
mod naive_commitment_test;

/// A trait for using commitment schemes generically.
pub trait Commitment:
    AddAssign
    + SubAssign
    + Sized
    + Default
    + Clone
    + core::ops::Neg<Output = Self>
    + Eq
    + core::ops::Sub<Output = Self>
    + core::fmt::Debug
    + core::marker::Sync
    + core::marker::Send
{
    /// The associated scalar that the commitment is for.
    /// There are multiple possible commitment schemes for a scalar, but only one scalar for any commitment.
    type Scalar: Scalar
        + for<'a> core::ops::Mul<&'a Self, Output = Self>
        + core::ops::Mul<Self, Output = Self>
        + serde::Serialize
        + for<'a> serde::Deserialize<'a>;

    /// The public setup for the commitment scheme.
    type PublicSetup<'a>;

    /// Compute the commitments for the given columns.
    ///
    /// The resulting commitments are written to the slice in `commitments`, which is a buffer.
    /// `commitments` is expected to have the same length as `committable_columns` and the behavior is undefined if it does not.
    /// The length of each [`CommittableColumn`] should be the same.
    ///
    /// `offset` is the amount that `committable_columns` is "offset" by. Logically adding `offset` many 0s to the beginning of each of the `committable_columns`.
    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self>;

    /// Converts the commitment to bytes that will be appended to the transcript.
    ///
    /// This is also useful for serialization purposes.
    fn to_transcript_bytes(&self) -> Vec<u8>;
}

mod commitment_evaluation_proof;
pub use commitment_evaluation_proof::CommitmentEvaluationProof;

#[cfg(test)]
pub(crate) mod commitment_evaluation_proof_test;
