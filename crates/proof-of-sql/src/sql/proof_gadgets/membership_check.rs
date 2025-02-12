use crate::{
    base::{
        database::{join_util::get_multiplicities, Column},
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
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
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal
/// or if the number of columns is zero.
pub(crate) fn first_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FirstRoundBuilder<'a, S>,
    alloc: &'a Bump,
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
) -> &'a [i128] {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    assert!(
        !columns.is_empty(),
        "The number of source columns should be greater than 0"
    );
    let multiplicities = get_multiplicities::<S>(candidate_subset, columns, alloc);
    builder.produce_intermediate_mle(multiplicities as &[_]);
    multiplicities
}

/// Perform final round evaluation of the membership check.
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal
/// or if the number of columns is zero.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn final_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    chi_n: &'a [bool],
    chi_m: &'a [bool],
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
) -> &'a [i128] {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    assert!(
        !columns.is_empty(),
        "The number of source columns should be greater than 0"
    );
    let multiplicities = get_multiplicities::<S>(candidate_subset, columns, alloc);

    // Fold the columns
    let c_fold = alloc.alloc_slice_fill_copy(chi_n.len(), Zero::zero());
    fold_columns(c_fold, alpha, beta, columns);
    let d_fold = alloc.alloc_slice_fill_copy(chi_m.len(), Zero::zero());
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

    // c_star + c_fold * c_star - chi_n = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - chi_m = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_m as &[_])]),
        ],
    );
    multiplicities
}

#[allow(dead_code, clippy::similar_names)]
pub(crate) fn verify_membership_check<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    chi_n_eval: S,
    chi_m_eval: S,
    column_evals: &[S],
    candidate_evals: &[S],
) -> Result<S, ProofError> {
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

    // c_star + c_fold * c_star - chi_n = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * c_fold_eval) * c_star_eval - chi_n_eval,
        2,
    )?;

    // d_star + d_fold * d_star - chi_m = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * d_fold_eval) * d_star_eval - chi_m_eval,
        2,
    )?;

    Ok(multiplicity_eval)
}
