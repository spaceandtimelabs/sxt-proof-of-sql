use super::{
    rand_F_tensors, rand_G_vecs, test_rng, ExtendedProverState, G1Projective, G2Projective,
    PublicParameters,
};
use crate::base::polynomial::compute_evaluation_vector;
use ark_ec::{pairing::Pairing, VariableBaseMSM};

#[test]
pub fn we_can_create_an_extended_verifier_state_from_an_extended_prover_state() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::test_rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    for nu in 0..max_nu {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
        let mut s1 = vec![Default::default(); 1 << nu];
        let mut s2 = vec![Default::default(); 1 << nu];
        compute_evaluation_vector(&mut s1, &s1_tensor);
        compute_evaluation_vector(&mut s2, &s2_tensor);
        let extended_prover_state = ExtendedProverState::new_from_tensors(
            s1_tensor.clone(),
            s2_tensor.clone(),
            v1.clone(),
            v2.clone(),
            nu,
        );
        assert_eq!(extended_prover_state.s1, s1);
        assert_eq!(extended_prover_state.s2, s2);
        let extended_verifier_state = extended_prover_state.calculate_verifier_state(&prover_setup);

        let C = Pairing::multi_pairing(&v1, &v2);
        let D_1 = Pairing::multi_pairing(&v1, prover_setup.Gamma_2[nu]);
        let D_2 = Pairing::multi_pairing(prover_setup.Gamma_1[nu], &v2);
        let E_1 = G1Projective::msm_unchecked(&v1, &s2);
        let E_2 = G2Projective::msm_unchecked(&v2, &s1);

        assert_eq!(extended_verifier_state.base_state.C, C);
        assert_eq!(extended_verifier_state.base_state.D_1, D_1);
        assert_eq!(extended_verifier_state.base_state.D_2, D_2);
        assert_eq!(extended_verifier_state.base_state.nu, nu);
        assert_eq!(extended_verifier_state.E_1, E_1);
        assert_eq!(extended_verifier_state.E_2, E_2);
        assert_eq!(extended_verifier_state.s1_tensor, s1_tensor);
        assert_eq!(extended_verifier_state.s2_tensor, s2_tensor);
    }
}
