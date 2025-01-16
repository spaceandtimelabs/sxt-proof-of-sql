use snafu::Snafu;

#[derive(Snafu, Debug)]
/// These errors occur when a proof failed to verify.
pub enum ProofError {
    #[snafu(display("Verification error: {error}"))]
    /// This error occurs when a proof failed to verify.
    VerificationError { error: &'static str },
    /// This error occurs when a query plan is not supported.
    #[snafu(display("Unsupported query plan: {error}"))]
    UnsupportedQueryPlan { error: &'static str },
    /// This error occurs the type coercion of the result table failed.
    #[snafu(display("Result does not match query: type mismatch"))]
    InvalidTypeCoercion,
    /// This error occurs when the field names of the result table do not match the query.
    #[snafu(display("Result does not match query: field names mismatch"))]
    FieldNamesMismatch,
    /// This error occurs when the number of fields in the result table does not match the query.
    #[snafu(display("Result does not match query: field count mismatch"))]
    FieldCountMismatch,
    #[snafu(transparent)]
    ProofSizeMismatch { source: ProofSizeMismatch },
}

#[derive(Snafu, Debug)]
/// These errors occur when the proof size does not match the expected size.
pub enum ProofSizeMismatch {
    /// This error occurs when the sumcheck proof doesn't have enough coefficients.
    #[snafu(display("Sumcheck proof is too small"))]
    SumcheckProofTooSmall,
    /// This error occurs when the proof has too few MLE evaluations.
    #[snafu(display("Proof has too few MLE evaluations"))]
    TooFewMLEEvaluations,
    /// This error occurs when the number of post result challenges in the proof plan doesn't match the number specified in the proof
    #[snafu(display("Post result challenge count mismatch"))]
    PostResultCountMismatch,
    /// This error occurs when the number of constraints in the proof plan doesn't match the number specified in the proof
    #[snafu(display("Constraint count mismatch"))]
    ConstraintCountMismatch,
    /// This error occurs when the proof has too few bit distributions.
    #[snafu(display("Proof has too few bit distributions"))]
    TooFewBitDistributions,
    /// This error occurs when the proof has too few one lengths.
    #[snafu(display("Proof has too few one lengths"))]
    TooFewOneLengths,
    /// This error occurs when the proof has too few rho lengths.
    #[snafu(display("Proof has too few rho lengths"))]
    TooFewRhoLengths,
    /// This error occurs when the proof has too few sumcheck variables.
    #[snafu(display("Proof has too few sumcheck variables"))]
    TooFewSumcheckVariables,
}
