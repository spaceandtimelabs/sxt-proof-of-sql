use datafusion::{
    arrow::error::{ArrowError, Result as ArrowResult},
    common::{DataFusionError, Result as DataFusionResult},
};
use std::sync::{LockResult, PoisonError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofError {
    /// This error occurs when a proof failed to verify.
    #[error("verification error")]
    VerificationError,
    /// This error occurs when the proof encoding is malformed.
    #[error("Proof data could not be parsed.")]
    FormatError,
    /// This error occurs when there is no proof.
    #[error("Proof data could not be found.")]
    NoProofError,
    /// This error occurs when a certain DateType, PhysicalExpr or ExecutionPlan is not supported.
    #[error("Proof hasn't been implemented yet")]
    UnimplementedError,
    /// This error occurs when there are type incompatibilities e.g. wrapping a NegativeExpr into ColumnWrapper
    #[error("Type incompatibility found in inputs.")]
    TypeError,
    /// This error occurs when proofs are attempted on a raw unevaluated PhysicalExpr
    #[error("A PhysicalExpr can not be proven unless evaluated first")]
    UnevaluatedError,
    /// This error occurs when proofs are attempted on a raw unexecuted pExecutionPlan
    #[error("An ExecutionPlan can not be proven unless executed first")]
    UnexecutedError,
    /// This error occurs when a nullable array is used in a function/method that requires nonnull inputs
    #[error("Input array must not have nulls.")]
    NullabilityError,
    /// Poison error
    #[error("RwLock poisoned.")]
    PoisonError,
    /// Arrow error
    #[error("Arrow error found.")]
    ArrowError(ArrowError),
    /// Datafusion error
    #[error("Datafusion error found.")]
    DataFusionError(DataFusionError),
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

impl From<ArrowError> for ProofError {
    fn from(e: ArrowError) -> Self {
        ProofError::ArrowError(e)
    }
}

impl From<DataFusionError> for ProofError {
    fn from(e: DataFusionError) -> Self {
        ProofError::DataFusionError(e)
    }
}

impl<T> IntoProofResult<T> for LockResult<T> {
    fn into_proof_result(self) -> ProofResult<T> {
        self.map_err(|_| ProofError::PoisonError)
    }
}

impl<T> IntoProofResult<T> for ArrowResult<T> {
    fn into_proof_result(self) -> ProofResult<T> {
        self.map_err(ProofError::ArrowError)
    }
}

impl<T> IntoProofResult<T> for DataFusionResult<T> {
    fn into_proof_result(self) -> ProofResult<T> {
        self.map_err(ProofError::DataFusionError)
    }
}

// Convert numerous non-DF results to DataFusionResult
pub trait IntoDataFusionResult<T> {
    fn into_datafusion_result(self) -> DataFusionResult<T>;
}

impl<T> IntoDataFusionResult<T> for LockResult<T> {
    fn into_datafusion_result(self) -> DataFusionResult<T> {
        self.map_err(|e| DataFusionError::External(Box::new(ProofError::from(e))))
    }
}
