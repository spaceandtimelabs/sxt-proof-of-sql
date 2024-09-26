use super::{
    dory_reduce_prove, dory_reduce_verify, scalar_product_prove, scalar_product_verify,
    DoryMessages, ProverSetup, ProverState, VerifierSetup, VerifierState,
};
use merlin::Transcript;

/// This is the prover side of the Dory-Innerproduct algorithm in section 3.3 of <https://eprint.iacr.org/2020/1274.pdf>.
/// This function builds/enqueues `messages`, appends to `transcript`, and consumes `state`.
#[cfg(test)]
pub fn dory_inner_product_prove(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    mut state: ProverState,
    setup: &ProverSetup,
) {
    let nu = state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        dory_reduce_prove(messages, transcript, &mut state, setup);
    }
    scalar_product_prove(messages, transcript, state);
}

/// This is the verifier side of the Dory-Innerproduct algorithm in section 3.3 of <https://eprint.iacr.org/2020/1274.pdf>.
/// This function consumes/dequeues from `messages`, appends to `transcript`, and consumes `state`.
#[cfg(test)]
pub fn dory_inner_product_verify(
    messages: &mut DoryMessages,
    transcript: &mut Transcript,
    mut state: VerifierState,
    setup: &VerifierSetup,
) -> bool {
    let nu = state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        if !dory_reduce_verify(messages, transcript, &mut state, setup) {
            return false;
        }
    }
    scalar_product_verify(messages, transcript, state, setup)
}
