use crate::{
    base::{commitment::Commitment, proof::ProofError, scalar::Scalar},
    sql::proof::{ProofBuilder, VerificationBuilder},
};
use ark_std::rand;
use core::hash::Hash;
use std::iter::Sum;

/// Evaluates the range check of scalar values by converting each scalar into
/// a word array and processing it through a proof builder. This function
/// targets zero-copy commitment computation when converting from [Scalar] to
/// word-sized targets.
///
/// # Safety
/// This function converts scalar values (`Scalar`) to word slices.
/// ensures that:
/// - The data alignment of `u64` (from which the word slices are derived) is
///   sufficient for `u8`, ensuring proper memory alignment and access safety.
/// - Only the first 31 words of each `u64` array are accessed, aligning with the
///   cryptographic goal to prove that these words are within a specific numerical
///   range, namely [0, (p - 1)/2] or [0, 2^248 - 1].
/// - The `expr` slice must live at least as long as `'a` to ensure that references
///   to the data remain valid throughout the function's execution.
pub fn prover_evaluate_range_check<'a, S: Scalar + Hash>(
    _builder: &mut ProofBuilder<'a, S>,
    scalars: &'a [S],
) {
    // Convert scalars to word slices where each byte is converted into a scalar of type `S`
    let word_refs: Vec<Vec<S>> = scalars
        .iter()
        .map(|s| unsafe {
            let scalar_array: [u64; 4] = (*s).into();
            let scalar_bytes: &[u8] = std::slice::from_raw_parts(
                scalar_array.as_ptr() as *const u8,
                scalar_array.len() * std::mem::size_of::<u64>(),
            );
            // Now convert each byte to `S`
            scalar_bytes.iter().map(|&b| S::from(b)).collect()
        })
        .collect();

    // Convert scalars to word slices where each byte is converted into a scalar of type `S`,
    // and then invert each scalar, mapping `None` to `S::ZERO`.
    let _inverted_word_refs: Vec<Vec<S>> = scalars
        .iter()
        .map(|s| unsafe {
            // Convert scalar `s` into an array of u64
            let scalar_array: [u64; 4] = (*s).into();
            // Convert the array of u64 into a slice of bytes
            let scalar_bytes: &[u8] = std::slice::from_raw_parts(
                scalar_array.as_ptr() as *const u8,
                scalar_array.len() * std::mem::size_of::<u64>(),
            );
            // Convert each byte to `S` and then attempt to invert it
            scalar_bytes
                .iter()
                .map(|&b| S::from(b).inv().unwrap_or(S::ZERO))
                .collect()
        })
        .collect();

    // always non-degenerate so no need to worry about division by zero
    let alpha: S = S::rand(&mut rand::thread_rng());

    // create position labels from 0 to the length of scalars
    let n: Vec<S> = (0..scalars.len() as u32).map(|i| S::from(i)).collect();

    // collect the results of calculating 1 / (n[i] + alpha)
    let inverse_n_plus_alpha: Vec<S> = n
        .iter()
        .map(|&n| match (n + alpha).inv() {
            None => S::ZERO,
            Some(inverse) => inverse,
        })
        .collect();

    // A really lousy way to get the total count of each word occurance per row
    let counts = count_words_naive(&word_refs);

    // Calculate (word  + alpha) * inverse_word_plus_alpha_rows[i] - 1 = 0
    let _inverse_count_times_count_plus_alpha_constraints =
        inverse_n_times_n_plus_alpha_constraints(scalars, &inverse_n_plus_alpha, alpha);

    // For each word in a row, calculate 1 / (word + alpha)
    let inverse_word_plus_alpha_rows = get_inverse_words_plus_alpha_rows(&word_refs, alpha);

    // Calculate (word  + alpha) * inverse_word_plus_alpha_rows[i] - 1 = 0
    let _inverse_word_plus_alpha_constraints =
        get_inverse_word_plus_alpha_constraints(&word_refs, &inverse_word_plus_alpha_rows, alpha);

    // Calculate (word[i] + word[1] + word[2] ... + word[n]) - (count * 1 / (count + alpha))
    let _get_word_row_sum_minus_counts_times_n_inv_constraints =
        get_word_row_sum_minus_counts_times_n_inv(
            &inverse_word_plus_alpha_rows,
            &counts,
            &inverse_n_plus_alpha,
        );
}

// A really awful way of counting how many times each word appears in the total decomposition
// across all scalars
fn count_words_naive<S: PartialEq + Clone + From<u32>>(word_refs: &Vec<Vec<S>>) -> Vec<S> {
    let mut counts: Vec<S> = Vec::new();

    for row in word_refs {
        for word in row {
            // Count how many times `word` appears in all rows
            let mut count = 0;
            for check_row in word_refs {
                for check_item in check_row {
                    if word == check_item {
                        count += 1;
                    }
                }
            }
            counts.push(S::from(count)); // Convert count to S and push it
        }
    }

    counts
}
fn get_word_row_sum_minus_counts_times_n_inv<S: Scalar + Sum + Clone>(
    inverse_word_plus_alpha_rows: &[Vec<S>],
    counts: &[S],
    inverse_n: &[S],
) -> S {
    inverse_word_plus_alpha_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let row_sum: S = row.iter().cloned().sum(); // Sum of all words in the row
            let constraint: S = counts[i] * inverse_n[i];
            row_sum - constraint // Compute the constraint for this row
        })
        .sum() // Sum all computed constraints into a single scalar
}

fn inverse_n_times_n_plus_alpha_constraints<S: Scalar>(
    expr: &[S],
    inverse_n: &[S],
    alpha: S,
) -> Vec<S> {
    (0..expr.len() as u64)
        .zip(inverse_n.iter())
        .map(|(i, &transformed)| transformed * (S::from(i as u32) + alpha) - S::ONE)
        .collect()
}

fn get_inverse_word_plus_alpha_constraints<S: Scalar>(
    word_refs: &[Vec<S>],
    inverse_word_plus_alpha_rows: &[Vec<S>],
    alpha: S,
) -> Vec<Vec<S>> {
    word_refs
        .iter()
        .zip(inverse_word_plus_alpha_rows.iter())
        .map(|(word_row, transformed_row)| {
            word_row
                .iter()
                .zip(transformed_row)
                .map(|(&word, &transformed)| transformed * (word + alpha) - S::ONE)
                .collect::<Vec<S>>()
        })
        .collect()
}

fn get_inverse_words_plus_alpha_rows<S: Scalar>(word_refs: &[Vec<S>], alpha: S) -> Vec<Vec<S>> {
    let inverse_word_plus_alpha_rows: Vec<Vec<S>> = word_refs
        .iter()
        .map(|slice| {
            slice
                .iter()
                .map(|&word| match (word + alpha).inv() {
                    None => S::ZERO,
                    Some(inverse) => inverse,
                })
                .collect()
        })
        .collect();
    inverse_word_plus_alpha_rows
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
