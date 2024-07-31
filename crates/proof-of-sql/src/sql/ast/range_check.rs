use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar},
    sql::proof::{ProofBuilder, VerificationBuilder},
};

/// Evaluates the range check of scalar values by converting each scalar into
/// a byte array and processing it through a proof builder. This function
/// targets zero-copy commitment computation when converting from [Scalar] to
/// word-sized targets.
///
/// # Safety
/// This function converts scalar values (`Scalar`) to byte slices in a manner
/// that avoids unnecessary copying, using direct memory access. The conversion
/// ensures that:
/// - The data alignment of `u64` (from which the byte slices are derived) is
///   sufficient for `u8`, ensuring proper memory alignment and access safety.
/// - Only the first 31 bytes of each `u64` array are accessed, aligning with the
///   cryptographic goal to prove that these bytes are within a specific numerical
///   range, namely [0, (p - 1)/2] or [0, 2^248 - 1].
/// - The `expr` slice must live at least as long as `'a` to ensure that references
///   to the data remain valid throughout the function's execution.
pub fn prover_evaluate_range_check<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    expr: &'a [S],
) {
    let byte_refs: Vec<&'a [u8]> = expr
        .iter()
        .map(|s| unsafe {
            // Convert Scalar to [u64; 4] and then to &[u8]
            let scalar_array: [u64; 4] = (*s).into(); // Using `Into` trait to convert Scalar directly
            let scalar_bytes: &[u8] = std::slice::from_raw_parts(
                scalar_array.as_ptr() as *const u8,
                32, // Each u64 is 8 bytes, so [u64; 4] is 32 bytes
            );
            &scalar_bytes[..31] // Take the first 31 bytes
        })
        .collect();

    // Processing each byte reference with the builder
    for &byte_ref in &byte_refs {
        builder.produce_intermediate_mle(byte_ref);
    }
}

/// Evaluates a polynomial at a specified point to verify if the result matches
/// a given expression value. This function applies Horner's method for efficient
/// polynomial evaluation.
///
/// The function first retrieves the necessary coefficients from a
/// [VerificationBuilder] and then evaluates the polynomial. If the evaluated
/// result matches the given `expr_eval`, it confirms the validity of the
/// expression; otherwise, it raises an error.
///
/// # Type Parameters
/// * `C` - Represents a commitment type that must support basic arithmetic
///   operations (`Add`, `Mul`) and can be constructed from `u128`.
///
/// # Returns
/// * `Ok(())` if the computed polynomial value matches `expr_eval`.
/// * `Err(ProofError)` if there is a mismatch, indicating a verification failure.
pub fn verifier_evaluate_range_check<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    expr_eval: C::Scalar,
) -> Result<(), ProofError> {
    let mut word_columns_evals: Vec<C::Scalar> = Vec::with_capacity(30);

    for _ in 0..30 {
        let mle = builder.consume_intermediate_mle();
        word_columns_evals.push(mle);
    }

    let base: C::Scalar = C::Scalar::from(256);
    let mut accumulated = word_columns_evals[0];

    for eval in word_columns_evals.iter().skip(1) {
        accumulated = accumulated * base + *eval;
    }

    if expr_eval == accumulated {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "Computed polynomial does not match the evaluation expression.",
        ))
    }
}
