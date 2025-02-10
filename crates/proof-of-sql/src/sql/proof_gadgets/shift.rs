use crate::{
    base::{proof::ProofError, scalar::Scalar, slice_ops},
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

/// Perform first round evaluation of downward shift.
pub(crate) fn first_round_evaluate_shift<S: Scalar>(
    builder: &mut FirstRoundBuilder<'_, S>,
    num_rows: usize,
) {
    // Note that we don't produce one eval lengths here
    // since it needs to be done in uniqueness check which uses shifts.
    builder.produce_rho_evaluation_length(num_rows);
    builder.produce_rho_evaluation_length(num_rows + 1);
}

/// Perform final round evaluation of downward shift.
///
/// # Panics
/// Panics if `column.len() != shifted_column.len() - 1` which should always hold for shifts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn final_round_evaluate_shift<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    column: &'a [S],
    shifted_column: &'a [S],
) {
    let num_rows = column.len();
    assert_eq!(
        num_rows + 1,
        shifted_column.len(),
        "Shifted column length mismatch"
    );
    let rho_plus_chi_n =
        alloc.alloc_slice_fill_with(num_rows, |i| S::from(i as u64 + 1_u64)) as &[_];
    let rho_n_plus_1 = alloc.alloc_slice_fill_with(num_rows + 1, |i| S::from(i as u64)) as &[_];
    let chi_n_plus_1 = alloc.alloc_slice_fill_copy(num_rows + 1, true);

    let c_fold = alloc.alloc_slice_fill_copy(num_rows, Zero::zero());
    fold_columns(c_fold, alpha, beta, &[rho_plus_chi_n, column]);
    let c_fold_extended = alloc.alloc_slice_fill_copy(num_rows + 1, Zero::zero());
    c_fold_extended[..num_rows].copy_from_slice(c_fold);
    let c_star = alloc.alloc_slice_copy(c_fold_extended);
    slice_ops::add_const::<S, S>(c_star, One::one());
    slice_ops::batch_inversion(c_star);

    let d_fold = alloc.alloc_slice_fill_copy(num_rows + 1, Zero::zero());
    fold_columns(d_fold, alpha, beta, &[rho_n_plus_1, shifted_column]);
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

    // c_star + c_fold * c_star - chi_n_plus_1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_fold_extended as &[_]), Box::new(c_star as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n_plus_1 as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - chi_n_plus_1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_fold as &[_]), Box::new(d_star as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n_plus_1 as &[_])]),
        ],
    );
}

pub(crate) fn verify_shift<S: Scalar, B: VerificationBuilder<S>>(
    builder: &mut B,
    alpha: S,
    beta: S,
    column_eval: S,
    shifted_column_eval: S,
    chi_n_eval: S,
    chi_n_plus_1_eval: S,
) -> Result<(), ProofError> {
    let rho_n_eval = builder.try_consume_rho_evaluation()?;
    let rho_n_plus_1_eval = builder.try_consume_rho_evaluation()?;
    let c_fold_eval = alpha * fold_vals(beta, &[rho_n_eval + chi_n_eval, column_eval]);
    let d_fold_eval = alpha * fold_vals(beta, &[rho_n_plus_1_eval, shifted_column_eval]);
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    //sum c_star - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval - d_star_eval,
        1,
    )?;

    // c_star + c_fold * c_star - chi_n_plus_1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        c_star_eval + c_fold_eval * c_star_eval - chi_n_plus_1_eval,
        2,
    )?;

    // d_star + d_fold * d_star - chi_n_plus_1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_star_eval + d_fold_eval * d_star_eval - chi_n_plus_1_eval,
        2,
    )?;

    Ok(())
}
