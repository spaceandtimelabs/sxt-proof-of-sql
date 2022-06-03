use ark_poly::MultilinearExtension;

use crate::base::polynomial::{to_ark_scalar, DenseMultilinearExtension};
use crate::pip::sumcheck::{ProverState, ProverMessage, VerifierMessage};

#[allow(unused_variables)]
pub fn prove_round(
    prover_state: &mut ProverState,
    v_msg: &Option<VerifierMessage>,
) -> ProverMessage {
    if let Some(msg) = v_msg {
        if prover_state.round == 0 {
            panic!("first round should be prover first.");
        }
        prover_state.randomness.push(msg.randomness);
    
        // fix argument
        let i = prover_state.round;
        let r = prover_state.randomness[i - 1];
        for multiplicand in prover_state.flattened_ml_extensions.iter_mut() {
            *multiplicand = DenseMultilinearExtension{
                ark_impl: multiplicand.ark_impl.fix_variables(&[to_ark_scalar(&r)]),
            };
        }
    } else {
        if prover_state.round > 0 {
            panic!("verifier message is empty");
        }
    }
    ProverMessage{
        evaluations: Vec::with_capacity(0),
    }
}
