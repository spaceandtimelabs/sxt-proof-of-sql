use super::{rand_G_vecs, test_rng, ProverSetup, ProverState};
use ark_ec::pairing::Pairing;

#[test]
pub fn we_can_create_a_verifier_state_from_a_prover_state() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_nu, Gamma_2_nu) = rand_G_vecs(nu, &mut rng);
        let prover_state = ProverState::new(v1.clone(), v2.clone(), nu);
        let setup = ProverSetup::new(&Gamma_1_nu, &Gamma_2_nu, nu);
        let verifier_state = prover_state.calculate_verifier_state(&setup);

        let C = Pairing::multi_pairing(&v1, &v2);
        let D_1 = Pairing::multi_pairing(&v1, &Gamma_2_nu);
        let D_2 = Pairing::multi_pairing(&Gamma_1_nu, &v2);

        assert_eq!(verifier_state.C, C);
        assert_eq!(verifier_state.D_1, D_1);
        assert_eq!(verifier_state.D_2, D_2);
        assert_eq!(verifier_state.nu, nu);
    }
}
