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
/// This function converts scalar values (`Scalar`) to byte slices.
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
    scalars: &'a [S],
) {
    // Convert scalars to byte slices
    let byte_refs: Vec<&'a [u8]> = scalars
        .iter()
        .map(|s| unsafe {
            let scalar_array: [u64; 4] = (*s).into();
            let scalar_bytes: &[u8] = std::slice::from_raw_parts(
                scalar_array.as_ptr() as *const u8,
                scalar_array.len() * std::mem::size_of::<u64>(),
            );
            scalar_bytes
        })
        .collect();

    // always non-degenerate so no need to worry about division by zero
    let alpha: u8 = 17;
    // create position labels from 0 to the length of scalars
    let n: Vec<u64> = (0..scalars.len() as u64).collect();

    // collect the results of calculating 1 / (count[i] + alpha)
    let inverse_count_plus_alpha: Vec<f64> =
        n.iter().map(|&n| 1.0 / (n as f64 + alpha as f64)).collect();

    // Get the total count of each byte occurance per row
    let counts = get_count_byte_occurances(scalars, &byte_refs);
    // Calculate (byte  + alpha) * inverse_byte_plus_alpha_rows[i] - 1 = 0
    let inverse_count_times_count_plus_alpha_constraints =
        inverse_n_times_n_plus_alpha_constraints(scalars, &inverse_count_plus_alpha, alpha);

    // For each byte in a row, calculate 1 / (byte + alpha)
    let inverse_byte_plus_alpha_rows = get_inverse_bytes_plus_alpha_rows(&byte_refs, alpha);

    // Calculate (byte  + alpha) * inverse_byte_plus_alpha_rows[i] - 1 = 0
    let inverse_byte_plus_alpha_constraints =
        get_inverse_byte_plus_alpha_constraints(byte_refs, &inverse_byte_plus_alpha_rows, alpha);

    // Calculate (byte[i] + byte[1] + byte[2] ... + byte[n]) - (count * 1 / (count + alpha))
    let get_byte_row_sum_minus_counts_times_n_inv_constraints =
        get_byte_row_sum_minus_counts_times_n_inv(
            inverse_byte_plus_alpha_rows,
            counts,
            inverse_count_plus_alpha,
        );
}

fn get_byte_row_sum_minus_counts_times_n_inv(
    inverse_byte_plus_alpha_rows: Vec<Vec<f64>>,
    counts: Vec<u64>,
    inverse_n: Vec<f64>,
) -> f64 {
    inverse_byte_plus_alpha_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let row_sum: f64 = row.iter().sum(); // Sum of all bytes in the row
            let adjustment: f64 = counts[i] as f64 * inverse_n[i];
            row_sum - adjustment // Compute the constraint for this row
        })
        .sum() // Sum all computed constraints into a single f64
}

fn inverse_n_times_n_plus_alpha_constraints<'a, S: Scalar>(
    expr: &[S],
    inverse_n: &Vec<f64>,
    alpha: u8,
) -> Vec<f64> {
    (0..expr.len() as u64)
        .zip(inverse_n.iter())
        .map(|(i, &transformed)| transformed * ((i + alpha as u64) as f64) - 1.0)
        .collect()
}

fn get_inverse_byte_plus_alpha_constraints(
    byte_refs: Vec<&[u8]>,
    inverse_byte_plus_alpha_rows: &[Vec<f64>],
    alpha: u8,
) -> Vec<Vec<f64>> {
    byte_refs
        .iter()
        .zip(inverse_byte_plus_alpha_rows.iter())
        .map(|(byte_row, transformed_row)| {
            byte_row
                .iter()
                .zip(transformed_row)
                .map(|(&byte, &transformed)| transformed * ((byte + alpha) as f64) - 1.0)
                .collect::<Vec<f64>>()
        })
        .collect()
}

fn get_inverse_bytes_plus_alpha_rows(byte_refs: &[&[u8]], alpha: u8) -> Vec<Vec<f64>> {
    let inverse_byte_plus_alpha_rows: Vec<Vec<f64>> = byte_refs
        .iter()
        .map(|slice| {
            slice
                .iter()
                .map(|&byte| 1.0 / ((byte as f64) + (alpha as f64)))
                .collect()
        })
        .collect();
    inverse_byte_plus_alpha_rows
}

fn get_count_byte_occurances<'a, S: Scalar>(expr: &[S], byte_refs: &[&[u8]]) -> Vec<u64> {
    // Initialize the byte count vector with zeros
    let mut counts: Vec<u64> = vec![0; expr.len()];

    // Count occurrences of each byte value corresponding to the position labels
    for &bytes in byte_refs {
        for &byte in bytes {
            if (byte as usize) < counts.len() {
                counts[byte as usize] += 1;
            }
        }
    }
    counts
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
    let mut word_columns_evals: Vec<C::Scalar> = Vec::with_capacity(31);
    for _ in 0..31 {
        let mle = builder.consume_intermediate_mle();
        word_columns_evals.push(mle);
    }

    let base: C::Scalar = C::Scalar::from(256);
    let mut accumulated = word_columns_evals[0];

    for eval in word_columns_evals.iter() {
        accumulated = accumulated * base + *eval;
    }

    dbg!(expr_eval);
    dbg!(accumulated);

    if expr_eval == accumulated {
        Ok(())
    } else {
        Err(ProofError::VerificationError(
            "Computed polynomial does not match the evaluation expression.",
        ))
    }
}
