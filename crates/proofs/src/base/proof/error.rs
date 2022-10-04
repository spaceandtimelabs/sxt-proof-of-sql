use std::sync::{LockResult, PoisonError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofError {
    /// This error occurs when a proof failed to verify.
    #[error("Verification error")]
    VerificationError,
    /// This error occurs when the proof encoding is malformed.
    #[error("Proof data could not be parsed.")]
    FormatError,
    /// This error occurs when there is no proof.
    #[error("Proof data could not be found.")]
    NoProofError,
    /// This error occurs when a certain DateType, PhysicalExpr or ExecutionPlan is not supported.
    #[error("Proof hasn't been implemented yet.")]
    UnimplementedError,
    /// This error occurs when there are type incompatibilities e.g. wrapping a NegativeExpr into ColumnWrapper
    #[error("Type incompatibility found in inputs.")]
    TypeError,
    /// This error occurs when attempt is made to create a table with different columns having different lengths
    /// or no lengths given/deduced at all
    #[error(
        "Columns in a Table do not have the same length or the length can not be found/deduced."
    )]
    TableColumnLengthError,
    /// This error occurs when proofs are attempted on a raw unevaluated PhysicalExpr
    #[error("A PhysicalExpr can not be proven unless evaluated first.")]
    UnevaluatedError,
    /// This error occurs when proofs are attempted on a raw unexecuted ExecutionPlan
    #[error("An ExecutionPlan can not be proven unless executed first.")]
    UnexecutedError,
    /// This error occurs when a nullable array is used in a function/method that requires nonnull inputs
    #[error("Input array must not have nulls.")]
    NullabilityError,
    /// Poison error
    #[error("RwLock poisoned.")]
    PoisonError,
    /// A compressed Ristretto point could not be decompressed.
    #[error("A compressed Ristretto point could not be decompressed.")]
    DecompressionError,
    /// General error especially internal errors that shouldn't happen ever
    #[error("General error found.")]
    GeneralError,
}

pub type ProofResult<T> = std::result::Result<T, ProofError>;

// Convert numerous external results to ProofResult
pub trait IntoProofResult<T> {
    fn into_proof_result(self) -> ProofResult<T>;
}

impl<T> From<PoisonError<T>> for ProofError {
    fn from(_e: PoisonError<T>) -> Self {
        ProofError::PoisonError
    }
}

impl<T> IntoProofResult<T> for LockResult<T> {
    fn into_proof_result(self) -> ProofResult<T> {
        self.map_err(|_| ProofError::PoisonError)
    }
}
