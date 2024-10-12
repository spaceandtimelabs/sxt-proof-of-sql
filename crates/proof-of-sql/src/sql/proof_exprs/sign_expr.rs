use super::{
    is_within_acceptable_range, verify_constant_abs_decomposition,
    verify_constant_sign_decomposition,
};
use crate::{
    base::{
        bit::{compute_varying_bit_matrix, BitDistribution},
        commitment::Commitment,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType,
        VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;

/// Count the number of components needed to prove a sign decomposition
pub fn count_sign(builder: &mut CountBuilder) -> Result<(), ProofError> {
    let dist = builder.consume_bit_distribution()?;
    if !is_within_acceptable_range(&dist) {
        return Err(ProofError::VerificationError {
            error: "bit distribution outside of acceptable range",
        });
    }
    if dist.num_varying_bits() == 0 {
        return Ok(());
    }
    builder.count_intermediate_mles(dist.num_varying_bits());
    builder.count_subpolynomials(dist.num_varying_bits());
    builder.count_degree(3);
    if dist.has_varying_sign_bit() && dist.num_varying_bits() > 1 {
        builder.count_subpolynomials(1);
    }
    Ok(())
}

/// Compute the sign bit for a column of scalars.
///
/// # Panics
/// Panics if `bits.last()` is `None` or if `result.len()` does not match `table_length`.
///
/// todo! make this more efficient and targeted at just the sign bit rather than all bits to create a proof
pub fn result_evaluate_sign<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, expr.len());
    // bit_distribution
    let dist = BitDistribution::new::<S, _>(expr);

    // handle the constant case
    if dist.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    // prove that the bits are binary
    let bits = compute_varying_bit_matrix(alloc, expr, &dist);
    if !dist.has_varying_sign_bit() {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    let result = bits.last().unwrap();
    assert_eq!(table_length, result.len());
    result
}

/// Prove the sign decomposition for a column of scalars.
///
/// # Panics
/// Panics if `bits.last()` is `None`.
///
/// If x1, ..., xn denotes the data, prove the column of
/// booleans, i.e. sign bits, s1, ..., sn where si == 1 if xi > MID and
/// `si == 1` if `xi <= MID` and `MID` is defined in `base/bit/abs_bit_mask.rs`
///
/// Note: We can only prove the sign bit for non-zero scalars, and we restict
/// the range of non-zero scalar so that there is a unique sign representation.
pub fn prover_evaluate_sign<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
    #[cfg(test)] treat_column_of_zeros_as_negative: bool,
) -> &'a [bool] {
    let table_length = expr.len();
    // bit_distribution
    let dist = BitDistribution::new::<S, _>(expr);
    #[cfg(test)]
    let dist = {
        let mut dist = dist;
        if treat_column_of_zeros_as_negative && dist.vary_mask == [0; 4] {
            dist.or_all[3] = 1 << 63;
        }
        dist
    };
    builder.produce_bit_distribution(dist.clone());

    // handle the constant case
    if dist.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    // prove that the bits are binary
    let bits = compute_varying_bit_matrix(alloc, expr, &dist);
    prove_bits_are_binary(builder, &bits);
    if !dist.has_varying_sign_bit() {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    if dist.num_varying_bits() > 1 {
        prove_bit_decomposition(builder, alloc, expr, &bits, &dist);
    }

    // This might panic if `bits.last()` returns `None`.
    bits.last().unwrap()
}

/// Verify the sign decomposition for a column of scalars.
///
/// # Panics
/// Panics if `bit_evals.last()` is `None`.
///
/// See [`prover_evaluate_sign`].
pub fn verifier_evaluate_sign<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    eval: C::Scalar,
    one_eval: C::Scalar,
) -> Result<C::Scalar, ProofError> {
    // bit_distribution
    let dist = builder.consume_bit_distribution();
    let num_varying_bits = dist.num_varying_bits();

    // extract evaluations and commitmens of the multilinear extensions for the varying
    // bits of the expression
    let mut bit_evals = Vec::with_capacity(num_varying_bits);
    for _ in 0..num_varying_bits {
        let eval = builder.consume_intermediate_mle();
        bit_evals.push(eval);
    }

    // establish that the bits are binary
    verify_bits_are_binary(builder, &bit_evals);

    // handle the special case of the sign bit being constant
    if !dist.has_varying_sign_bit() {
        return verifier_const_sign_evaluate(&dist, eval, one_eval, &bit_evals);
    }

    // handle the special case of the absolute part being constant
    if dist.num_varying_bits() == 1 {
        verify_constant_abs_decomposition(&dist, eval, one_eval, bit_evals[0])?;
    } else {
        verify_bit_decomposition(builder, eval, &bit_evals, &dist);
    }

    Ok(*bit_evals.last().unwrap())
}

fn verifier_const_sign_evaluate<S: Scalar>(
    dist: &BitDistribution,
    eval: S,
    one_eval: S,
    bit_evals: &[S],
) -> Result<S, ProofError> {
    verify_constant_sign_decomposition(dist, eval, one_eval, bit_evals)?;
    if dist.sign_bit() {
        Ok(one_eval)
    } else {
        Ok(S::zero())
    }
}

fn prove_bits_are_binary<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    bits: &[&'a [bool]],
) {
    for &seq in bits {
        builder.produce_intermediate_mle(seq);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(seq)]),
                (-S::one(), vec![Box::new(seq), Box::new(seq)]),
            ],
        );
    }
}

fn verify_bits_are_binary<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    bit_evals: &[C::Scalar],
) {
    for bit_eval in bit_evals {
        builder.produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            *bit_eval - *bit_eval * *bit_eval,
        );
    }
}

/// # Panics
/// Panics if `bits.last()` returns `None`.
///
/// This function generates subpolynomial terms for sumcheck, involving the scalar expression and its bit decomposition.
fn prove_bit_decomposition<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
    bits: &[&'a [bool]],
    dist: &BitDistribution,
) {
    let sign_mle = bits.last().unwrap();
    let sign_mle: &[_] =
        alloc.alloc_slice_fill_with(sign_mle.len(), |i| 1 - 2 * i32::from(sign_mle[i]));
    let mut terms: Vec<SumcheckSubpolynomialTerm<S>> = Vec::new();

    // expr
    terms.push((S::one(), vec![Box::new(expr)]));

    // expr bit decomposition
    let const_part = S::from_limbs(dist.constant_part());
    if !const_part.is_zero() {
        terms.push((-const_part, vec![Box::new(sign_mle)]));
    }
    let mut vary_index = 0;
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        terms.push((
            -S::from_limbs(mult),
            vec![Box::new(sign_mle), Box::new(bits[vary_index])],
        ));
        vary_index += 1;
    });
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomialType::Identity, terms);
}

/// # Panics
/// Panics if `bit_evals.last()` returns `None`.
///
/// This function checks the consistency of the bit evaluations with the expression evaluation.
fn verify_bit_decomposition<C: Commitment>(
    builder: &mut VerificationBuilder<'_, C>,
    expr_eval: C::Scalar,
    bit_evals: &[C::Scalar],
    dist: &BitDistribution,
) {
    let mut eval = expr_eval;
    let sign_eval = bit_evals.last().unwrap();
    let sign_eval = builder.mle_evaluations.input_one_evaluation - C::Scalar::TWO * *sign_eval;
    let mut vary_index = 0;
    eval -= sign_eval * C::Scalar::from_limbs(dist.constant_part());
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        let bit_eval = bit_evals[vary_index];
        eval -= C::Scalar::from_limbs(mult) * sign_eval * bit_eval;
        vary_index += 1;
    });
    builder.produce_sumcheck_subpolynomial_evaluation(SumcheckSubpolynomialType::Identity, eval);
}
