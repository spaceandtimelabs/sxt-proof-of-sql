use super::{
    rand_F_vecs, rand_G_vecs, rand_Hs, test_rng, ExtendedProverState, ProverSetup, G1, G2,
};
use ark_ec::{pairing::Pairing, ScalarMul, VariableBaseMSM};

#[test]
pub fn we_can_create_an_extended_verifier_state_from_an_extended_prover_state() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (Gamma_1_nu, Gamma_2_nu) = rand_G_vecs(nu, &mut rng);
        let (H_1, H_2) = rand_Hs(&mut rng);
        let extended_prover_state =
            ExtendedProverState::new(s1.clone(), s2.clone(), v1.clone(), v2.clone(), nu);
        let setup = ProverSetup::new(&Gamma_1_nu, &Gamma_2_nu, nu, H_1, H_2);
        let extended_verifier_state = extended_prover_state.calculate_verifier_state(&setup);

        let C = Pairing::multi_pairing(&v1, &v2);
        let D_1 = Pairing::multi_pairing(&v1, &Gamma_2_nu);
        let D_2 = Pairing::multi_pairing(&Gamma_1_nu, &v2);
        let E_1 = G1::msm(&ScalarMul::batch_convert_to_mul_base(&v1), &s2).unwrap();
        let E_2 = G2::msm(&ScalarMul::batch_convert_to_mul_base(&v2), &s1).unwrap();

        assert_eq!(extended_verifier_state.base_state.C, C);
        assert_eq!(extended_verifier_state.base_state.D_1, D_1);
        assert_eq!(extended_verifier_state.base_state.D_2, D_2);
        assert_eq!(extended_verifier_state.base_state.nu, nu);
        assert_eq!(extended_verifier_state.E_1, E_1);
        assert_eq!(extended_verifier_state.E_2, E_2);
        assert_eq!(extended_verifier_state.s1, s1);
        assert_eq!(extended_verifier_state.s2, s2);
    }
}
