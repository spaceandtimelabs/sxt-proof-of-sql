use super::{
    eval_vmv_re_prove, eval_vmv_re_verify, test_rng, DoryMessages, PublicParameters, F, GT, VMV,
};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn we_can_prove_and_verify_an_eval_vmv_re() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let extended_prover_state =
        eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    assert_eq!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup,
        ),
        Some(extended_prover_state.calculate_verifier_state(&prover_setup)),
    );
}

#[test]
fn we_can_prove_and_verify_an_eval_vmv_re_for_multiple_nu_values() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::test_rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();

    for nu in 0..max_nu {
        let vmv = VMV::rand(nu, &mut rng);
        let prover_state = vmv.calculate_prover_state(&prover_setup);
        let verifier_state = vmv.calculate_verifier_state(&prover_setup);

        let mut transcript = Transcript::new(b"eval_vmv_re_test");
        let mut messages = DoryMessages::default();
        let extended_prover_state =
            eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

        let mut transcript = Transcript::new(b"eval_vmv_re_test");
        assert_eq!(
            eval_vmv_re_verify(
                &mut messages,
                &mut transcript,
                verifier_state,
                &verifier_setup
            ),
            Some(extended_prover_state.calculate_verifier_state(&prover_setup)),
        );
    }
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_a_message_is_modified() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let extended_prover_state =
        eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    assert_ne!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        )
        .unwrap(),
        extended_prover_state.calculate_verifier_state(&prover_setup),
    );
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_there_are_too_few_GT_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let _ = eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages.pop();

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    assert_eq!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ),
        None
    );
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_there_are_too_few_G1_messages() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let _ = eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.G1_messages.pop();

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    assert_eq!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ),
        None
    );
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_the_setups_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let pp_wrong = PublicParameters::test_rand(nu, &mut rng);
    let verifier_setup = (&pp_wrong).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let extended_prover_state =
        eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");

    assert_ne!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        )
        .unwrap(),
        extended_prover_state.calculate_verifier_state(&prover_setup),
    );
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_the_commitment_is_wrong() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let mut verifier_state = vmv.calculate_verifier_state(&prover_setup);

    verifier_state.T = GT::rand(&mut rng).into();

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let extended_prover_state =
        eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");

    assert_ne!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        )
        .unwrap(),
        extended_prover_state.calculate_verifier_state(&prover_setup),
    );
}

#[test]
fn we_fail_to_verify_an_eval_vmv_re_when_the_evaluation_value_is_wrong() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let vmv = VMV::rand(nu, &mut rng);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let mut verifier_state = vmv.calculate_verifier_state(&prover_setup);

    verifier_state.y = F::rand(&mut rng);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");
    let mut messages = DoryMessages::default();
    let extended_prover_state =
        eval_vmv_re_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"eval_vmv_re_test");

    assert_ne!(
        eval_vmv_re_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        )
        .unwrap(),
        extended_prover_state.calculate_verifier_state(&prover_setup),
    );
}
