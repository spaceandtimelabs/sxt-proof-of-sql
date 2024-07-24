use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar},
    sql::proof::{ProofBuilder, VerificationBuilder},
};

/// Evaluates the range check of scalar values by converting each scalar into 
/// a byte array and processing it through a proof builder. This function 
/// targets zero-copy commitment computation when converting from `Scalar` to
/// word-sized targets.
///
/// # Safety
/// This function uses `unsafe` to convert scalar values (`S`) represented as 
/// `[u64; 4]` arrays into byte (word) slices. It requires that data alignment of `u64` 
/// is sufficient for `u8`, and that the `expr` slice lives at least as long as 
/// `'a`. The conversion exposes native endianness, and only the first 31 bytes 
/// of the `u64` array are accessed because we are eventually trying to prove 
/// that the bytes are within the range [0, (p - 1)/2], or [0, 2^248 - 1].
pub fn prover_evaluate_range_check<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    expr: &'a [S],
) {
    let byte_refs: Vec<&'a [u8]> = expr
        .iter()
        .map(|&s| {
            let scalar_u64s: [u64; 4] = s.into();
            // Strategy: Use `unsafe` to zero-copy pointer cast from `[u64; 4]` to `&[u8]`.
            // Safety:
            // * Alignment: `u64` variables have stricter alignment requirements compared to `u8`.
            //    Therefore, a pointer to `u64` is guaranteed to be correctly aligned for `u8` access.
            // * This block assumes that the data in `expr` (which `scalar_u64s` references)
            //    lives long enough for the duration of this function's execution, matching or exceeding `'a`.
            // * Endianness: This conversion exposes the machine's native endianness. The bytes are accessed
            //    directly from the `u64` data structure, which means the byte order will reflect the machine's
            //    architecture (little-endian or big-endian).
            // * Data Integrity: Direct byte access via this method does not alter the original data.
            // Only the first 31 bytes of the `u64` array are used, avoiding any out-of-bounds access.
            unsafe {
                let scalar_bytes: &[u8] = std::slice::from_raw_parts(
                    scalar_u64s.as_ptr() as *const u8,
                    std::mem::size_of::<[u64; 4]>(),
                );
                std::slice::from_raw_parts(scalar_bytes.as_ptr(), 31)
            }
        })
        .collect();

    for &byte_ref in &byte_refs {
        builder.produce_intermediate_mle(byte_ref);
    }
}

/// Evaluates a polynomial at a specified point to verify if the result matches
/// a given expression value. This function applies Horner's method for efficient
/// polynomial evaluation.
///
/// The function first retrieves the necessary coefficients from a
/// `VerificationBuilder` and then evaluates the polynomial. If the evaluated
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
pub fn verifier_evaluate_range_check<
    C: Commitment<Scalar = C> + std::ops::Add<Output = C> + std::ops::Mul<Output = C> + From<u128>,
>(
    builder: &mut VerificationBuilder<C>,
    expr_eval: C::Scalar,
) -> Result<(), ProofError> {
    let mut word_columns_evals: Vec<C> = Vec::with_capacity(30);

    // Consume intermediate values from the builder
    for _ in 0..30 {
        let mle = builder.consume_intermediate_mle();
        word_columns_evals.push(mle);
    }

    let base: C = C::from(256);
    let mut accumulated = word_columns_evals[0];

    // Horner's method reformulates the polynomial evaluation process to
    // minimize the number of multiplications:
    // P(x) = (...((aₙx + aₙ₋₁)x + aₙ₋₂)x + ... + a₁)x + a₀
    // This expression is evaluated at x = 256.
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
