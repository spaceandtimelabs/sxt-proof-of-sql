use super::{
    AppendColumnCommitmentsError, ColumnCommitments,
    ColumnCommitmentsMismatch, Commitment, DuplicateIdentifiers,
};
use core::ops::Range;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Cannot create a [`TableCommitment`] with a negative range.
#[derive(Debug, Snafu)]
#[snafu(display("cannot create a TableCommitment with a negative range"))]
pub struct NegativeRange;

/// Cannot create a [`TableCommitment`] from columns of mixed length.
#[derive(Debug, Snafu)]
#[snafu(display("cannot create a TableCommitment from columns of mixed length"))]
pub struct MixedLengthColumns;

/// Errors that can occur when trying to create or extend a [`TableCommitment`] from columns.
#[derive(Debug, Snafu)]
pub enum TableCommitmentFromColumnsError {
    /// Cannot construct [`TableCommitment`] from columns of mixed length.
    #[snafu(transparent)]
    MixedLengthColumns {
        /// The underlying source error
        source: MixedLengthColumns,
    },
    /// Cannot construct [`TableCommitment`] from columns with duplicate identifiers.
    #[snafu(transparent)]
    DuplicateIdentifiers {
        /// The underlying source error
        source: DuplicateIdentifiers,
    },
}

/// Errors that can occur when attempting to append rows to a [`TableCommitment`].
#[derive(Debug, Snafu)]
pub enum AppendTableCommitmentError {
    /// Cannot append columns of mixed length to existing [`TableCommitment`].
    #[snafu(transparent)]
    MixedLengthColumns {
        /// The underlying source error
        source: MixedLengthColumns,
    },
    /// Encountered error when appending internal [`ColumnCommitments`].
    #[snafu(transparent)]
    AppendColumnCommitments {
        /// The underlying source error
        source: AppendColumnCommitmentsError,
    },
}

/// Errors that can occur when performing arithmetic on [`TableCommitment`]s.
#[derive(Debug, Snafu)]
pub enum TableCommitmentArithmeticError {
    /// Cannot perform arithmetic on columns with mismatched metadata.
    #[snafu(transparent)]
    ColumnMismatch {
        /// The underlying source error
        source: ColumnCommitmentsMismatch,
    },
    /// Cannot perform [`TableCommitment`] arithmetic that would result in a negative range.
    #[snafu(transparent)]
    NegativeRange {
        /// The underlying source error
        source: NegativeRange,
    },
    /// Cannot perform arithmetic for noncontiguous table commitments.
    #[snafu(display(
        "cannot perform table commitment arithmetic for noncontiguous table commitments"
    ))]
    NonContiguous,
}

/// Commitment for an entire table, with column and table metadata.
///
/// Unlike [`ColumnCommitments`], all columns in this commitment must have the same length.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableCommitment<C>
where
    C: Commitment,
{
    pub (crate) column_commitments: ColumnCommitments<C>,
    pub (crate) range: Range<usize>,
}

