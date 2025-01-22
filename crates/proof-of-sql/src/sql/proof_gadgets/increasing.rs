//! Prove that a column is non-strictly increasing.
use super::{
    final_round_evaluate_shift, first_round_evaluate_shift, prover_evaluate_sign,
    verifier_evaluate_sign, verify_shift,
};
use crate::{
    base::{proof::ProofError, scalar::Scalar},
    sql::proof::{FinalRoundBuilder, FirstRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;

/// Perform first round evaluation of increasing.
pub(crate) fn first_round_evaluate_increasing<S: Scalar>(
    builder: &mut FirstRoundBuilder<'_, S>,
    num_rows: usize,
) {
    builder.produce_one_evaluation_length(num_rows + 1);
    first_round_evaluate_shift(builder, num_rows);
}

/// Perform final round evaluation of increasing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn final_round_evaluate_increasing<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    column: &'a [S],
) {
    let num_rows = column.len();
    let shifted_column =
        alloc.alloc_slice_fill_with(
            num_rows + 1,
            |i| {
                if i == 0 {
                    S::ZERO
                } else {
                    column[i - 1]
                }
            },
        );
    builder.produce_intermediate_mle(shifted_column as &[_]);
    // 1. Prove that `shifted_column` is a shift of `column`
    final_round_evaluate_shift(builder, alloc, alpha, beta, column, shifted_column);
    // 2. Construct an indicator `diff` such that if `diff` is all nonnegative except for the first and last elements,
    // then `column` is non-strictly increasing
    let diff = alloc.alloc_slice_fill_with(num_rows + 1, |i| {
        if i == num_rows {
            -column[num_rows - 1]
        } else {
            column[i] - shifted_column[i]
        }
    });

    // 3. Prove that `diff` is all nonnegative with the possible exception of the first and last elements
    // sign(diff) == 0 for all but the first element and the last element
    // The first and last elements can only fit into three patterns
    // 1. negative and non-negative
    // 2. non-negative and negative
    // 3. negative and negative
    // Hence the evaluation of sign has to be in one of three cases
    // 1. singleton_one_eval
    // 2. shifted_one_eval - one_eval
    // 3. singleton_one_eval + shifted_one_eval - one_eval
    prover_evaluate_sign(builder, alloc, diff);
}

pub(crate) fn verify_increasing<S: Scalar>(
    builder: &mut VerificationBuilder<S>,
    alpha: S,
    beta: S,
    column_eval: S,
    one_eval: S,
) -> Result<(), ProofError> {
    // 1. Verify that `shifted_column` is a shift of `column`
    let shifted_column_eval = builder.try_consume_final_round_mle_evaluation()?;
    let shifted_one_eval = builder.try_consume_one_evaluation()?;
    verify_shift(
        builder,
        alpha,
        beta,
        column_eval,
        shifted_column_eval,
        one_eval,
        shifted_one_eval,
    )?;
    // 2. Verify that `sign_eval` is correct, that is, `column` is strictly increasing.
    // The first and last elements of `diff` can only fit into three patterns
    // 1. negative and non-negative
    // 2. non-negative and negative
    // 3. negative and negative
    // Hence the evaluation of sign has to be in one of three cases
    // 1. singleton_one_eval
    // 2. shifted_one_eval - one_eval
    // 3. singleton_one_eval + shifted_one_eval - one_eval
    let sign_eval =
        verifier_evaluate_sign(builder, column_eval - shifted_column_eval, shifted_one_eval)?;
    let singleton_one_eval = builder.mle_evaluations.singleton_one_evaluation;
    let last_element_eval = shifted_one_eval - one_eval;
    if sign_eval != singleton_one_eval
        && sign_eval != last_element_eval
        && sign_eval != singleton_one_eval + last_element_eval
    {
        return Err(ProofError::VerificationError {
            error: "column is not non-strictly increasing",
        });
    }
    Ok(())
}
