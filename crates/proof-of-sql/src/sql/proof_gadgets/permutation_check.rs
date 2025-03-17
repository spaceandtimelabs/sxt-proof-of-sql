use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_plans::{fold_columns, fold_vals},
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use num_traits::{One, Zero};

/// Perform final round evaluation of the permutation check.
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal
/// or if the number of columns is zero.
pub(crate) fn final_round_evaluate_permutation_check<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    chi: &'a [bool],
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
) {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    assert!(
        !columns.is_empty(),
        "The number of source columns should be greater than 0"
    );
    // Fold the columns
    let c_fold = alloc.alloc_slice_fill_copy(chi.len(), Zero::zero());
    fold_columns(c_fold, alpha, beta, columns);
    let d_fold = alloc.alloc_slice_fill_copy(chi.len(), Zero::zero());
    fold_columns(d_fold, alpha, beta, candidate_subset);

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

    // c_star + c_fold * c_star - chi = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi as &[_])]),
        ],
    );
}

pub(crate) fn verify_permutation_check<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    chi_eval: S,
    column_evals: &[S],
    candidate_evals: &[S],
) -> Result<(), ProofError> {
    // Check that the source and candidate columns have the same amount of columns
    if column_evals.len() != candidate_evals.len() {
        return Err(ProofError::VerificationError {
            error: "The number of source and candidate columns should be equal",
        });
    }
    let c_fold_eval = fold_vals(beta, column_evals);
    let d_fold_eval = fold_vals(beta, candidate_evals);
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    // sum c_star - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval - d_star_eval,
        1,
    )?;

    // c_star + c_fold * c_star - chi = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * c_fold_eval) * c_star_eval - chi_eval,
        2,
    )?;

    // d_star + d_fold * d_star - chi = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (S::ONE + alpha * d_fold_eval) * d_star_eval - chi_eval,
        2,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{final_round_evaluate_permutation_check, verify_permutation_check};
    use crate::{
        base::{
            database::table_utility::borrowed_bigint,
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder},
    };
    use bumpalo::Bump;
    use std::collections::VecDeque;

    #[test]
    fn we_can_do_permutation_check() {
        let alloc = Bump::new();
        let column = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
        let candidate_table = borrowed_bigint::<TestScalar>("c", [2, 3, 1], &alloc).1;
        let mut final_round_builder: FinalRoundBuilder<TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        final_round_evaluate_permutation_check(
            &mut final_round_builder,
            &alloc,
            TestScalar::TWO,
            TestScalar::TEN,
            &[true, true, true],
            &[column],
            &[candidate_table],
        );
        let verification_builder = run_verify_for_each_row(
            3,
            &final_round_builder,
            3,
            |verification_builder, chi_eval, evaluation_point| {
                verify_permutation_check(
                    verification_builder,
                    TestScalar::TWO,
                    TestScalar::TEN,
                    chi_eval,
                    &[column.inner_product(evaluation_point)],
                    &[candidate_table.inner_product(evaluation_point)],
                )
                .unwrap();
            },
        );
        assert!(verification_builder
            .get_identity_results()
            .iter()
            .all(|v| v.iter().all(|val| *val)));
        assert!(verification_builder
            .get_zero_sum_results()
            .iter()
            .all(|v| *v));
    }
}
