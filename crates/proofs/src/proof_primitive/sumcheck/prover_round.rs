/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use ark_ff::{One, Zero};
use ark_poly::MultilinearExtension;
use curve25519_dalek::scalar::Scalar;

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
        let i = prover_state.round;
        let r = prover_state.randomness[i - 1];
        for multiplicand in prover_state.flattened_ml_extensions.iter_mut() {
            *multiplicand = DenseMultilinearExtension {
                ark_impl: multiplicand.ark_impl.fix_variables(&[to_ark_scalar(&r)]),
            };
        }
    } else if prover_state.round > 0 {
        panic!("verifier message is empty");
    }

    prover_state.round += 1;

    if prover_state.round > prover_state.num_vars {
        panic!("Prover is not active");
    }

    let i = prover_state.round;
    let nv = prover_state.num_vars;
    let degree = prover_state.max_multiplicands; // the degree of univariate polynomial sent by prover at this round

    let mut products_sum = Vec::with_capacity(degree + 1);
    products_sum.resize(degree + 1, Scalar::zero());

    // generate sum
    for b in 0..1 << (nv - i) {
        let mut t_as_field = ArkScalar::zero();
        for scalar in products_sum.iter_mut().take(degree + 1) {
            // evaluate P_round(t)
            for (coefficient, products) in &prover_state.list_of_products {
                let num_multiplicands = products.len();
                let mut product = to_ark_scalar(coefficient);
                for multiplicand in products.iter().take(num_multiplicands) {
                    let table = &prover_state.flattened_ml_extensions[*multiplicand].ark_impl; // j's range is checked in init
                    multiply_product_by_term(table, b, &t_as_field, &mut product)
                }
                *scalar += from_ark_scalar(&product);
            }
            t_as_field += ArkScalar::one();
        }
    }

    products_sum
}

fn multiply_product_by_term(
    table: &ark_poly::DenseMultilinearExtension<ArkScalar>,
    b: usize,
    t_as_field: &ArkScalar,
    product: &mut ArkScalar,
) {
    let term = table[b << 1] * (ArkScalar::one() - t_as_field) + table[(b << 1) + 1] * t_as_field;
    *product *= term;
}
