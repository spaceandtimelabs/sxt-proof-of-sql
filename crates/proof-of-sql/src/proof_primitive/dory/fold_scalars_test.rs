use super::{
    extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs, fold_scalars_0_prove,
    fold_scalars_0_verify, rand_F_tensors, rand_G_vecs, test_rng, DoryMessages,
    ExtendedProverState, PublicParameters,
};
use merlin::Transcript;

#[test]
fn we_can_fold_scalars() {
    let mut rng = test_rng();
    let nu = 0;
    let pp = PublicParameters::rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"fold_scalars_test");
    let mut messages = DoryMessages::default();
    let prover_folded_state =
        fold_scalars_0_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"fold_scalars_test");
    let verifier_folded_state = fold_scalars_0_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs,
    );
    assert_eq!(
        prover_folded_state.calculate_verifier_state(&prover_setup),
        verifier_folded_state
    );
}
