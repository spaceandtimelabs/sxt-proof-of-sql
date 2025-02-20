use super::{ProverSetup, PublicParameters, VerifierSetup, test_rng};
use ark_ec::pairing::Pairing;
use std::{fs, path::Path};

#[test]
fn we_can_create_and_manually_check_a_small_prover_setup() {
    let mut rng = test_rng();
    let pp = PublicParameters::test_rand(2, &mut rng);
    let setup = ProverSetup::from(&pp);
    assert_eq!(setup.max_nu, 2);
    assert_eq!(setup.Gamma_1.len(), 3);
    assert_eq!(setup.Gamma_2.len(), 3);
    assert_eq!(setup.Gamma_1[0], pp.Gamma_1[0..1].to_vec());
    assert_eq!(setup.Gamma_1[1], pp.Gamma_1[0..2].to_vec());
    assert_eq!(setup.Gamma_1[2], pp.Gamma_1[0..4].to_vec());
    assert_eq!(setup.Gamma_2[0], pp.Gamma_2[0..1].to_vec());
    assert_eq!(setup.Gamma_2[1], pp.Gamma_2[0..2].to_vec());
    assert_eq!(setup.Gamma_2[2], pp.Gamma_2[0..4].to_vec());
    assert_eq!(setup.H_1, pp.H_1);
    assert_eq!(setup.H_2, pp.H_2);
    assert_eq!(setup.Gamma_2_fin, pp.Gamma_2_fin);
}

#[test]
fn we_can_create_save_load_and_manually_check_a_small_verifier_setup() {
    let mut rng = test_rng();
    let pp = PublicParameters::test_rand(2, &mut rng);
    let v_setup = VerifierSetup::from(&pp);

    v_setup.save_to_file(Path::new("setup.bin")).unwrap();
    let setup = VerifierSetup::load_from_file(Path::new("setup.bin"));
    assert!(setup.is_ok());

    let setup = setup.unwrap();
    assert_eq!(setup.max_nu, 2);
    assert_eq!(setup.Delta_1L.len(), 3);
    assert_eq!(setup.Delta_1R.len(), 3);
    assert_eq!(setup.Delta_2L.len(), 3);
    assert_eq!(setup.Delta_2R.len(), 3);
    assert_eq!(setup.chi.len(), 3);
    assert_eq!(
        setup.Delta_1L[1],
        Pairing::multi_pairing(&pp.Gamma_1[0..1], &pp.Gamma_2[0..1])
    );
    assert_eq!(
        setup.Delta_1L[2],
        Pairing::multi_pairing(&pp.Gamma_1[0..2], &pp.Gamma_2[0..2])
    );
    assert_eq!(
        setup.Delta_1R[1],
        Pairing::multi_pairing(&pp.Gamma_1[1..2], &pp.Gamma_2[0..1])
    );
    assert_eq!(
        setup.Delta_1R[2],
        Pairing::multi_pairing(&pp.Gamma_1[2..4], &pp.Gamma_2[0..2])
    );
    assert_eq!(
        setup.Delta_2L[1],
        Pairing::multi_pairing(&pp.Gamma_1[0..1], &pp.Gamma_2[0..1])
    );
    assert_eq!(
        setup.Delta_2L[2],
        Pairing::multi_pairing(&pp.Gamma_1[0..2], &pp.Gamma_2[0..2])
    );
    assert_eq!(
        setup.Delta_2R[1],
        Pairing::multi_pairing(&pp.Gamma_1[0..1], &pp.Gamma_2[1..2])
    );
    assert_eq!(
        setup.Delta_2R[2],
        Pairing::multi_pairing(&pp.Gamma_1[0..2], &pp.Gamma_2[2..4])
    );
    assert_eq!(
        setup.chi[0],
        Pairing::multi_pairing(&pp.Gamma_1[0..1], &pp.Gamma_2[0..1])
    );
    assert_eq!(
        setup.chi[1],
        Pairing::multi_pairing(&pp.Gamma_1[0..2], &pp.Gamma_2[0..2])
    );
    assert_eq!(
        setup.chi[2],
        Pairing::multi_pairing(&pp.Gamma_1[0..4], &pp.Gamma_2[0..4])
    );
    assert_eq!(setup.Gamma_1_0, pp.Gamma_1[0]);
    assert_eq!(setup.Gamma_2_0, pp.Gamma_2[0]);
    assert_eq!(setup.H_1, pp.H_1);
    assert_eq!(setup.H_2, pp.H_2);
    assert_eq!(setup.H_T, Pairing::pairing(pp.H_1, pp.H_2));
    assert_eq!(setup.Gamma_2_fin, pp.Gamma_2_fin);

    fs::remove_file(Path::new("setup.bin")).unwrap();
}

#[test]
fn we_can_create_prover_setups_with_various_sizes() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let setup = ProverSetup::from(&pp);
        assert_eq!(setup.Gamma_1.len(), nu + 1);
        assert_eq!(setup.Gamma_2.len(), nu + 1);
        for k in 0..=nu {
            assert_eq!(setup.Gamma_1[k].len(), 1 << k);
            assert_eq!(setup.Gamma_2[k].len(), 1 << k);
        }
        assert_eq!(setup.max_nu, nu);
        assert_eq!(setup.H_1, pp.H_1);
        assert_eq!(setup.H_2, pp.H_2);
        assert_eq!(setup.Gamma_2_fin, pp.Gamma_2_fin);
    }
}

#[test]
fn we_can_create_verifier_setups_with_various_sizes() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let setup = VerifierSetup::from(&pp);
        assert_eq!(setup.Delta_1L.len(), nu + 1);
        assert_eq!(setup.Delta_1R.len(), nu + 1);
        assert_eq!(setup.Delta_2L.len(), nu + 1);
        assert_eq!(setup.Delta_2R.len(), nu + 1);
        assert_eq!(setup.chi.len(), nu + 1);
        for k in 1..=nu {
            assert_eq!(setup.Delta_1L[k], setup.Delta_2L[k]);
            assert_ne!(setup.Delta_1L[k], setup.Delta_1R[k]);
            assert_ne!(setup.Delta_2L[k], setup.Delta_2R[k]);
            assert_ne!(setup.Delta_1R[k], setup.Delta_2R[k]);
            assert_ne!(setup.chi[k], setup.Delta_1L[k]);
            assert_ne!(setup.chi[k], setup.Delta_2L[k]);
            assert_ne!(setup.chi[k], setup.Delta_1R[k]);
            assert_ne!(setup.chi[k], setup.Delta_2R[k]);
        }
        assert_eq!(
            setup.chi[0],
            Pairing::pairing(setup.Gamma_1_0, setup.Gamma_2_0)
        );
        assert_eq!(setup.H_1, pp.H_1);
        assert_eq!(setup.H_2, pp.H_2);
        assert_eq!(setup.H_T, Pairing::pairing(pp.H_1, pp.H_2));
        assert_eq!(setup.Gamma_2_fin, pp.Gamma_2_fin);
    }
}

// nu size 1 = 3890 bytes
// nu size 2 = 6770 bytes
// nu size 3 = 9650 bytes
// nu size 4 = 12530 bytes
// nu size 5 = 15410 bytes
#[test]
fn we_can_serialize_and_deserialize_verifier_setups() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let setup = VerifierSetup::from(&pp);
        let serialized = postcard::to_allocvec(&setup).unwrap();
        let deserialized: VerifierSetup = postcard::from_bytes(&serialized).unwrap();
        assert_eq!(setup, deserialized);
    }
}
