/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use curve25519_dalek::scalar::Scalar;
use num_traits::Zero;
use rayon::prelude::*;

use crate::base::polynomial::{from_ark_scalar, ArkScalar, DenseMultilinearExtension};
use crate::base::scalar::ToArkScalar;
use crate::proof_primitive::sumcheck::ProverState;

#[tracing::instrument(
    name = "proofs.proof_primitive.sumcheck.prover_round.prove_round",
    level = "info",
    skip_all
)]
pub fn prove_round(prover_state: &mut ProverState, r_maybe: &Option<Scalar>) -> Vec<Scalar> {
    if let Some(r) = r_maybe {
        if prover_state.round == 0 {
            panic!("first round should be prover first.");
        }
        prover_state.randomness.push(*r);

        // fix argument
        let r_as_field = prover_state.randomness[prover_state.round - 1].to_ark_scalar();
        prover_state
            .flattened_ml_extensions
            .par_iter_mut()
            .for_each(|multiplicand| {
                in_place_fix_variable(multiplicand, r_as_field);
            });
    } else if prover_state.round > 0 {
        panic!("verifier message is empty");
    }

    prover_state.round += 1;

    if prover_state.round > prover_state.num_vars {
        panic!("Prover is not active");
    }

    let degree = prover_state.max_multiplicands; // the degree of univariate polynomial sent by prover at this round
    let round_length = 1usize << (prover_state.num_vars - prover_state.round);

    // The pseudocode of what this is trying to do is:

    // foreach t in 0..=degree compute
    //   sum over row in 0..round_length:
    //     sum over product in list_of_products:
    //       product over multiplicand in product:
    //         table = the mle of the multiplicand
    //         table[2b] * (1-t) + table[2b+1] * t
    // This gives a vector of length degree + 1

    // The order of these loops is changed for the purpose of efficiency.

    // The outer loop is the loop over all products in the list_of_products
    let result = prover_state
        .list_of_products
        .par_iter()
        .map(|(coefficient, multiplicand_indices)| {
            // The second loop is the loop over the row (b) in 0..round_length
            (0..round_length)
                .into_par_iter()
                .map(|b| {
                    // We add a vector of products, which takes a bit of extra memory. The reason for this is for the efficient modification described below
                    let mut products = vec![*coefficient; degree + 1];

                    // The third loop is the loop over the factors/multiplicand in the product term.
                    for &multiplicand_index in multiplicand_indices {
                        let table = &prover_state.flattened_ml_extensions[multiplicand_index];

                        // This third+final loop give an efficient way of computing
                        // products[t] *= table[b << 1] * (ArkScalar::one() - t_as_field) + table[(b << 1) + 1] * t_as_field;
                        // It requires only 1 addition (plus the cumulative multiplication) to accomplish the same task.
                        // It relies on the fact that
                        // table[b << 1] * (ArkScalar::one() - t_as_field) + table[(b << 1) + 1] * t_as_field == table[b << 1] + t * diff
                        let mut start = table[b << 1];
                        let step = table[(b << 1) + 1] - start;

                        // The innermost loop loops over the values (t) that we are evaluating at.
                        products.iter_mut().take(degree).for_each(|product| {
                            *product *= start;
                            start += step;
                        });
                        products[degree] *= start;
                    }
                    products
                })
                .reduce(|| vec![ArkScalar::zero(); degree + 1], vec_elementwise_add)
        })
        .reduce(|| vec![ArkScalar::zero(); degree + 1], vec_elementwise_add);

    result.into_iter().map(|a| from_ark_scalar(&a)).collect()
}

/// This is equivalent to
/// *multiplicand = DenseMultilinearExtension {
///                    ark_impl: multiplicand.ark_impl.fix_variables(&[r_as_field]),
///                };
/// Only it does it in place
fn in_place_fix_variable(multiplicand: &mut DenseMultilinearExtension, r_as_field: ArkScalar) {
    assert!(multiplicand.num_vars > 0, "invalid size of partial point");
    multiplicand.num_vars -= 1;
    for b in 0..(1 << multiplicand.num_vars) {
        let left: ArkScalar = multiplicand.evaluations[b << 1];
        let right: ArkScalar = multiplicand.evaluations[(b << 1) + 1];
        multiplicand.evaluations[b] = left + r_as_field * (right - left);
    }
}

fn vec_elementwise_add(a: Vec<ArkScalar>, b: Vec<ArkScalar>) -> Vec<ArkScalar> {
    a.into_iter()
        .zip(b)
        .map(|(x, y)| x + y)
        .collect::<Vec<ArkScalar>>()
}
