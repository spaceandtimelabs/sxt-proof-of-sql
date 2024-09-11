//! Types for creation and utilization of cryptographic commitments to proof-of-sql data.
use crate::base::scalar::Scalar;
#[cfg(feature = "blitzar")]
pub use blitzar::{
    compute::{init_backend, init_backend_with_config, BackendConfig},
    proof::InnerProductProof,
};
use core::ops::{AddAssign, SubAssign};
use curve25519_dalek::ristretto::RistrettoPoint;

mod committable_column;
pub(crate) use committable_column::CommittableColumn;

mod vec_commitment_ext;
pub use vec_commitment_ext::{NumColumnsMismatch, VecCommitmentExt};

mod column_bounds;
use super::scalar::Curve25519Scalar;
pub use column_bounds::{Bounds, ColumnBounds, NegativeBounds};

mod column_commitment_metadata;
pub use column_commitment_metadata::ColumnCommitmentMetadata;

mod column_commitment_metadata_map;
pub use column_commitment_metadata_map::{
    ColumnCommitmentMetadataMap, ColumnCommitmentMetadataMapExt, ColumnCommitmentsMismatch,
};

mod column_commitments;
pub use column_commitments::{
    AppendColumnCommitmentsError, ColumnCommitments, DuplicateIdentifiers,
};

mod table_commitment;
pub use table_commitment::{
    AppendTableCommitmentError, MixedLengthColumns, NegativeRange, TableCommitment,
    TableCommitmentArithmeticError, TableCommitmentFromColumnsError,
};

mod query_commitments;
pub use query_commitments::{QueryCommitments, QueryCommitmentsExt};

/// Module for providing a mock commitment.
pub mod naive_commitment;

/// Module for providing a test commitment evaluation proof.
pub mod test_evaluation_proof;

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
    fn compute_commitments(
        commitments: &mut [Self],
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    );
}

impl Commitment for RistrettoPoint {
    type Scalar = Curve25519Scalar;
    type PublicSetup<'a> = ();
    #[cfg(feature = "blitzar")]
    fn compute_commitments(
        commitments: &mut [Self],
        committable_columns: &[CommittableColumn],
        offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) {
        let sequences = Vec::from_iter(committable_columns.iter().map(Into::into));
        let mut compressed_commitments = vec![Default::default(); committable_columns.len()];
        blitzar::compute::compute_curve25519_commitments(
            &mut compressed_commitments,
            &sequences,
            offset as u64,
        );
        commitments
            .iter_mut()
            .zip(compressed_commitments.iter())
            .for_each(|(c, cc)| {
                *c = cc.decompress().expect(
                    "invalid ristretto point decompression in Commitment::compute_commitments",
                );
            });
    }
    #[cfg(not(feature = "blitzar"))]
    fn compute_commitments(
        _commitments: &mut [Self],
        _committable_columns: &[CommittableColumn],
        _offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) {
        unimplemented!()
    }
}

mod commitment_evaluation_proof;
pub use commitment_evaluation_proof::CommitmentEvaluationProof;
#[cfg(test)]
pub(crate) mod commitment_evaluation_proof_test;
