use super::{
    dory_inner_product_prove, dory_inner_product_verify, rand_G_vecs, rand_Hs, test_rng,
    DoryMessages, ProverSetup, ProverState, VerifierSetup, G1, GT,
};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn we_can_prove_and_verify_a_dory_inner_product() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_can_prove_and_verify_a_dory_inner_product_for_multiple_nu_values() {
    let mut rng = test_rng();
    let max_nu = 5;
    let (Gamma_1, Gamma_2) = rand_G_vecs(max_nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, max_nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, max_nu, H_1, H_2);

    for nu in 0..max_nu {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let prover_state = ProverState::new(v1, v2, nu);
        let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = Transcript::new(b"dory_inner_product_test");
        let mut messages = DoryMessages::default();
        dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

        let mut transcript = Transcript::new(b"dory_inner_product_test");
        assert!(dory_inner_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));
    }
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_a_message_is_modified() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_there_are_too_few_GT_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages.pop();

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_there_are_too_many_GT_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages.push(GT::rand(&mut rng));

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_there_are_too_few_G1_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.G1_messages.pop();

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_there_are_too_many_G1_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.G1_messages.push(G1::rand(&mut rng));

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_the_transcripts_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test_wrong");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_the_setups_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (Gamma_1_wrong, Gamma_2_wrong) = rand_G_vecs(nu, &mut rng);
    let (H_1_wrong, H_2_wrong) = rand_Hs(&mut rng);
    let verifier_setup =
        VerifierSetup::new(&Gamma_1_wrong, &Gamma_2_wrong, nu, H_1_wrong, H_2_wrong);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_the_commitment_is_wrong() {
    let mut rng = test_rng();
    let nu = 3;
    let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
    let (H_1, H_2) = rand_Hs(&mut rng);
    let prover_setup = ProverSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let verifier_setup = VerifierSetup::new(&Gamma_1, &Gamma_2, nu, H_1, H_2);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    verifier_state.C = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}
