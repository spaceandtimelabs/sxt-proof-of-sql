/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use ark_ff::One;
use ark_poly::MultilinearExtension;
use curve25519_dalek::scalar::Scalar;
use rayon::prelude::*;

use crate::base::polynomial::{
    from_ark_scalar, to_ark_scalar, ArkScalar, DenseMultilinearExtension,
};
use crate::proof_primitive::sumcheck::ProverState;

pub fn prove_round(prover_state: &mut ProverState, r_maybe: &Option<Scalar>) -> Vec<Scalar> {
    if let Some(r) = r_maybe {
        if prover_state.round == 0 {
            panic!("first round should be prover first.");
        }
        prover_state.randomness.push(*r);

        // fix argument
        let r_as_field = to_ark_scalar(&prover_state.randomness[prover_state.round - 1]);
        prover_state
            .flattened_ml_extensions
            .par_iter_mut()
            .for_each(|multiplicand| {
                *multiplicand = DenseMultilinearExtension {
                    ark_impl: multiplicand.ark_impl.fix_variables(&[r_as_field]),
                };
            });
    } else if prover_state.round > 0 {
        panic!("verifier message is empty");
    }

    prover_state.round += 1;

    if prover_state.round > prover_state.num_vars {
        panic!("Prover is not active");
    }

    let degree = prover_state.max_multiplicands; // the degree of univariate polynomial sent by prover at this round

    // generate sum
    (0..=degree)
        .into_par_iter()
        .map(|t| {
            let t_as_field = ArkScalar::from(t as u32);
            from_ark_scalar(
                &(0..1 << (prover_state.num_vars - prover_state.round))
                    .into_par_iter()
                    .map(|b| evaluate_p_round_of_t(prover_state, &t_as_field, b))
                    .sum::<ArkScalar>(),
            )
        })
        .collect()
}

fn evaluate_p_round_of_t(
    prover_state: &ProverState,
    t_as_field: &ArkScalar,
    b: usize,
) -> ArkScalar {
    prover_state
        // evaluate P_round(t)
        .list_of_products
        .iter()
        .map(|(coefficient, multiplicand_indices)| {
            evaluate_term(
                prover_state,
                coefficient,
                multiplicand_indices,
                t_as_field,
                b,
            )
        })
        .sum::<ArkScalar>()
}

fn evaluate_term(
    prover_state: &ProverState,
    coefficient: &Scalar,
    multiplicand_indices: &[usize],
    t_as_field: &ArkScalar,
    b: usize,
) -> ArkScalar {
    to_ark_scalar(coefficient)
        * multiplicand_indices
            // parallelizing this innermost iterator slows down the
            // benches significantly, despite the fact that rayon
            // uses work-stealing
            .iter()
            .map(|multiplicand_index| {
                evaluate_multiplicand(prover_state, *multiplicand_index, t_as_field, b)
            })
            .product::<ArkScalar>()
}

fn evaluate_multiplicand(
    prover_state: &ProverState,
    multiplicand_index: usize,
    t_as_field: &ArkScalar,
    b: usize,
) -> ArkScalar {
    let table = &prover_state.flattened_ml_extensions[multiplicand_index].ark_impl; // j's range is checked in init
    table[b << 1] * (ArkScalar::one() - t_as_field) + table[(b << 1) + 1] * t_as_field
}
