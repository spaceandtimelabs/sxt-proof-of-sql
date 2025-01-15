use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_plans::{fold_columns, fold_vals},
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use num_traits::{One, Zero};

/// Perform first round evaluation of the membership check.
pub(crate) fn first_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FirstRoundBuilder<'a, S>,
    multiplicities: &'a [i128],
) {
    builder.produce_intermediate_mle(multiplicities);
    builder.request_post_result_challenges(2);
}

/// Perform final round evaluation of the membership check.
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn final_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
    multiplicities: &'a [i128],
    input_ones: &'a [bool],
    candidate_ones: &'a [bool],
) {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    // Fold the columns
    let c_fold = alloc.alloc_slice_fill_copy(input_ones.len(), Zero::zero());
    fold_columns(c_fold, alpha, beta, columns);
    let d_fold = alloc.alloc_slice_fill_copy(candidate_ones.len(), Zero::zero());
    fold_columns(d_fold, alpha, beta, candidate_subset);

    let c_star = alloc.alloc_slice_copy(c_fold);
    slice_ops::add_const::<S, S>(c_star, One::one());
    slice_ops::batch_inversion(c_star);

    let d_star = alloc.alloc_slice_copy(d_fold);
    slice_ops::add_const::<S, S>(d_star, One::one());
    slice_ops::batch_inversion(d_star);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    // sum c_star * multiplicities - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(multiplicities as &[_])],
            ),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    // c_star + c_fold * c_star - input_ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(input_ones as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - candidate_ones = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(candidate_ones as &[_])]),
        ],
    );
}

#[allow(dead_code)]
pub(crate) fn verify_membership_check<S: Scalar>(
    builder: &mut VerificationBuilder<S>,
    alpha: S,
    beta: S,
    input_one_eval: S,
    candidate_one_eval: S,
    column_evals: &[S],
    candidate_evals: &[S],
) -> Result<(), ProofError> {
    // Check that the source and candidate columns have the same amount of columns
    if column_evals.len() != candidate_evals.len() {
        return Err(ProofError::VerificationError {
            error: "The number of source and candidate columns should be equal",
        });
    }
    let multiplicity_eval = builder.try_consume_first_round_mle_evaluation()?;
    let c_fold_eval = fold_vals(beta, column_evals);
    let d_fold_eval = fold_vals(beta, candidate_evals);
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    // sum c_star * multiplicities - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval * multiplicity_eval - d_star_eval,
        2,
    )?;

    // c_star + c_fold * c_star - input_ones = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * c_fold_eval) * c_star_eval - input_one_eval,
        2,
    )?;

    // d_star + d_fold * d_star - candidate_ones = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * d_fold_eval) * d_star_eval - candidate_one_eval,
        2,
    )?;

    Ok(())
}
