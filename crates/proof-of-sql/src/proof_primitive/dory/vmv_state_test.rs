use super::{test_rng, PublicParameters, F, VMV};
use ark_ec::pairing::Pairing;

#[test]
fn we_can_create_correct_vmv_states_from_a_small_fixed_vmv() {
    let mut rng = test_rng();
    let nu = 2;
    let pp = PublicParameters::rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let Gamma_1 = pp.Gamma_1.clone();
    let Gamma_2 = pp.Gamma_2.clone();
    let L = vec![100.into(), 101.into(), 102.into(), 103.into()];
    let R = vec![200.into(), 201.into(), 202.into(), 203.into()];
    let M = vec![
        vec![300.into(), 301.into(), 302.into(), 303.into()],
        vec![310.into(), 311.into(), 312.into(), 313.into()],
        vec![320.into(), 321.into(), 322.into(), 323.into()],
        vec![330.into(), 331.into(), 332.into(), 333.into()],
    ];
    let vmv = VMV::new(M, L.clone(), R.clone(), nu);
    let prover_state = vmv.calculate_prover_state(&prover_setup);
    let verifier_state = vmv.calculate_verifier_state(&prover_setup);

    assert_eq!(prover_state.nu, nu);
    assert_eq!(prover_state.L_vec, L);
    assert_eq!(prover_state.R_vec, R);
    assert_eq!(
        prover_state.T_vec_prime,
        vec![
            Gamma_1[0] * F::from(300)
                + Gamma_1[1] * F::from(301)
                + Gamma_1[2] * F::from(302)
                + Gamma_1[3] * F::from(303),
            Gamma_1[0] * F::from(310)
                + Gamma_1[1] * F::from(311)
                + Gamma_1[2] * F::from(312)
                + Gamma_1[3] * F::from(313),
            Gamma_1[0] * F::from(320)
                + Gamma_1[1] * F::from(321)
                + Gamma_1[2] * F::from(322)
                + Gamma_1[3] * F::from(323),
            Gamma_1[0] * F::from(330)
                + Gamma_1[1] * F::from(331)
                + Gamma_1[2] * F::from(332)
                + Gamma_1[3] * F::from(333),
        ]
    );
    assert_eq!(
        prover_state.v_vec,
        vec![
            (300 * 100 + 310 * 101 + 320 * 102 + 330 * 103).into(),
            (301 * 100 + 311 * 101 + 321 * 102 + 331 * 103).into(),
            (302 * 100 + 312 * 101 + 322 * 102 + 332 * 103).into(),
            (303 * 100 + 313 * 101 + 323 * 102 + 333 * 103).into()
        ]
    );
    // Because the VMV is not built from tensors, we can not check the `verifier_state.l_tensor` and `verifier_state.r_tensor`
    assert_eq!(
        verifier_state.T,
        Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(300)
            + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(301)
            + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(302)
            + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(303)
            + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(310)
            + Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(311)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(312)
            + Pairing::pairing(Gamma_1[3], Gamma_2[1]) * F::from(313)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(320)
            + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(321)
            + Pairing::pairing(Gamma_1[2], Gamma_2[2]) * F::from(322)
            + Pairing::pairing(Gamma_1[3], Gamma_2[2]) * F::from(323)
            + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(330)
            + Pairing::pairing(Gamma_1[1], Gamma_2[3]) * F::from(331)
            + Pairing::pairing(Gamma_1[2], Gamma_2[3]) * F::from(332)
            + Pairing::pairing(Gamma_1[3], Gamma_2[3]) * F::from(333)
    );
    assert_eq!(
        verifier_state.y,
        (300 * 100 * 200
            + 310 * 101 * 200
            + 320 * 102 * 200
            + 330 * 103 * 200
            + 301 * 100 * 201
            + 311 * 101 * 201
            + 321 * 102 * 201
            + 331 * 103 * 201
            + 302 * 100 * 202
            + 312 * 101 * 202
            + 322 * 102 * 202
            + 332 * 103 * 202
            + 303 * 100 * 203
            + 313 * 101 * 203
            + 323 * 102 * 203
            + 333 * 103 * 203)
            .into()
    );
}

#[test]
fn we_can_create_vmv_states_from_random_vmv_and_get_correct_sizes() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    for nu in 0..max_nu {
        let vmv = VMV::rand(nu, &mut rng);

        assert_eq!(vmv.L.len(), 1 << nu);
        assert_eq!(vmv.R.len(), 1 << nu);
        assert_eq!(vmv.M.len(), 1 << nu);
        for row in &vmv.M {
            assert_eq!(row.len(), 1 << nu);
        }

        let prover_state = vmv.calculate_prover_state(&prover_setup);
        let verifier_state = vmv.calculate_verifier_state(&prover_setup);

        assert_eq!(prover_state.nu, nu);
        assert_eq!(prover_state.L_vec, vmv.L);
        assert_eq!(prover_state.R_vec, vmv.R);
        assert_eq!(prover_state.l_tensor, vmv.l_tensor);
        assert_eq!(prover_state.r_tensor, vmv.r_tensor);
        assert_eq!(prover_state.T_vec_prime.len(), 1 << nu);
        assert_eq!(prover_state.v_vec.len(), 1 << nu);

        assert_eq!(verifier_state.l_tensor, vmv.l_tensor);
        assert_eq!(verifier_state.r_tensor, vmv.r_tensor);
    }
}
