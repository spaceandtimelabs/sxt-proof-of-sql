use super::{
    fold_scalars_0_prove, fold_scalars_0_verify, rand_F_vecs, rand_G_vecs, rand_Hs, test_rng,
    DoryMessages, ExtendedProverState, ProverSetup, VerifierSetup,
};
use merlin::Transcript;

#[test]
fn we_can_fold_scalars() {
    let mut rng = test_rng();
    let nu = 0;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (s1, s2) = rand_F_vecs(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new(s1, s2, v1, v2, nu);
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
    );
    assert_eq!(
        prover_folded_state.calculate_verifier_state(&prover_setup),
        verifier_folded_state
    );
}
