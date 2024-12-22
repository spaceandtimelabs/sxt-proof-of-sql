//! Ordering check for both unique and non-unique cases.
//! Currently we only support doing this check on a single column.
use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_plans::{fold_columns, fold_vals},
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use itertools::Itertools;
use num_traits::{One, Zero};

/// Perform final round evaluation of the ordering check.
#[allow(dead_code)]
pub(crate) fn final_round_evaluate_ordering<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    columns: &[Column<'a, S>],
    ordered_columns: &[Column<'a, S>],
    num_rows: usize,
) {
    // 1. Fold the columns
    let ones = alloc.alloc_slice_fill_copy(num_rows, true);
    let shifted_ones = alloc.alloc_slice_fill_copy(num_rows + 1, true);
    shifted_ones[0] = false;

    let c_fold = alloc.alloc_slice_fill_copy(num_rows, Zero::zero());
    fold_columns(c_fold, alpha, beta, columns);
    let d_fold = alloc.alloc_slice_fill_copy(num_rows, Zero::zero());
    fold_columns(d_fold, alpha, beta, ordered_columns);

    let c_star = alloc.alloc_slice_copy(c_fold);
    slice_ops::add_const::<S, S>(c_star, One::one());
    slice_ops::batch_inversion(c_star);

    let d_star = alloc.alloc_slice_copy(d_fold);
    slice_ops::add_const::<S, S>(d_star, One::one());
    slice_ops::batch_inversion(d_star);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    // sum c_star - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    // c_star + c_fold * c_star - ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(ones as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(ones as &[_])]),
        ],
    );
}

#[allow(dead_code)]
pub(crate) fn verify_ordering<S: Scalar>(
    builder: &mut VerificationBuilder<S>,
    alpha: S,
    beta: S,
    one_eval: S,
    shifted_one_eval: S,
    singleton_one_eval: S,
    column_evals: &[S],
    ordered_evals: &[S],
) -> Result<(), ProofError> {
    let c_fold_eval = alpha * fold_vals(beta, column_evals);
    let d_fold_eval = alpha * fold_vals(beta, ordered_evals);
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    // sum c_star - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval - d_star_eval,
        2,
    )?;

    // c_star + c_fold * c_star - ones = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        c_star_eval + c_fold_eval * c_star_eval - one_eval,
        2,
    )?;

    // d_star + d_fold * d_star - ones = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_star_eval + d_fold_eval * d_star_eval - one_eval,
        2,
    )?;

    Ok(())
}
