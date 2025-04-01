use alloc::vec::Vec;
use ark_bn254::G1Affine;
use ark_serialize::{CanonicalDeserialize, Compress, SerializationError, Validate};

/// When borrowed, `PublicSetup` type associated with the `HyperKZG` commitment scheme.
///
/// This "Owned" version is occasionally useful when actively allocating the setup to memory.
/// For example, deserialization, or generation.
///
/// See [`HyperKZGPublicSetup`] for the actual associated public setup type.
pub type HyperKZGPublicSetupOwned = Vec<G1Affine>;

/// `PublicSetup` type associated with the `HyperKZG` commitment scheme.
pub type HyperKZGPublicSetup<'a> = &'a [G1Affine];

const COMPRESSED_SIZE: usize = 32;

/// Deserialize a [`HyperKZGPublicSetupOwned`] from any `Read` implementor.
///
/// This expects the same format used in the proof-of-sql public ppot binaries. That is,
/// ark-serialized, compressed points flatly concatenated in the file (no length prefix).
#[cfg(feature = "std")]
pub fn deserialize_flat_compressed_hyperkzg_public_setup_from_reader<R: std::io::Read>(
    mut reader: R,
    validate: Validate,
) -> Result<HyperKZGPublicSetupOwned, SerializationError> {
    std::iter::repeat_with(|| {
        let mut buffer = [0; COMPRESSED_SIZE];
        let num_bytes_read = reader.read(&mut buffer);
        (buffer, num_bytes_read)
    })
    .map_while(|(bytes, num_bytes_read)| match num_bytes_read {
        Ok(0) => None, // EOF, end iterator
        Ok(_) => Some(G1Affine::deserialize_with_mode(
            &bytes[..],
            Compress::Yes,
            validate,
        )),
        Err(e) => Some(Err(e.into())),
    })
    .collect()
}

/// Deserialize a [`HyperKZGPublicSetupOwned`] from a byte slice.
///
/// This expects the same format used in the proof-of-sql public ppot binaries. That is,
/// ark-serialized, compressed points flatly concatenated in the file (no length prefix).
pub fn deserialize_flat_compressed_hyperkzg_public_setup_from_slice(
    bytes: &[u8],
    validate: Validate,
) -> Result<HyperKZGPublicSetupOwned, SerializationError> {
    bytes
        .chunks(COMPRESSED_SIZE)
        .map(|chunk| G1Affine::deserialize_with_mode(chunk, Compress::Yes, validate))
        .collect()
}

#[cfg(all(test, feature = "hyperkzg_proof"))]
#[must_use]
/// Load a small setup for testing.
/// This returns a public setup and a verifier key.
pub fn load_small_setup_for_testing() -> (
    HyperKZGPublicSetupOwned,
    nova_snark::provider::hyperkzg::VerifierKey<super::HyperKZGEngine>,
) {
    use super::HyperKZGEngine;
    use ark_ec::AffineRepr;
    use halo2curves::bn256::{Fq, Fq2, G1Affine, G2Affine};
    use nova_snark::{
        provider::hyperkzg::{CommitmentKey, EvaluationEngine},
        traits::evaluation::EvaluationEngineTrait,
    };

    const VK_X_REAL: [u64; 4] = [
        0x2a74_74c0_708b_ef80,
        0xf762_edcf_ecfe_1c73,
        0x2340_a37d_fae9_005f,
        0x285b_1f14_edd7_e663,
    ];
    const VK_X_IMAG: [u64; 4] = [
        0x85ad_b083_e48c_197b,
        0x39c2_b413_1094_5472,
        0xda72_7c1d_ef86_0103,
        0x17cc_9307_7f56_f654,
    ];
    const VK_Y_REAL: [u64; 4] = [
        0xc6db_5ddb_9bde_7fd0,
        0x0931_3450_580c_4c17,
        0x29ec_66e8_f530_f685,
        0x2bad_9a37_4aec_49d3,
    ];
    const VK_Y_IMAG: [u64; 4] = [
        0xa630_d3c7_cdaa_6ed9,
        0xe32d_d53b_1584_4956,
        0x674f_5b2f_6fdb_69d9,
        0x219e_dfce_ee17_23de,
    ];
    let tau_h = G2Affine {
        x: Fq2::new(Fq::from_raw(VK_X_REAL), Fq::from_raw(VK_X_IMAG)),
        y: Fq2::new(Fq::from_raw(VK_Y_REAL), Fq::from_raw(VK_Y_IMAG)),
    };
    let (_, vk) = EvaluationEngine::<HyperKZGEngine>::setup(&CommitmentKey::new(
        vec![],
        G1Affine::generator(),
        tau_h,
    ));

    let mut ps = super::deserialize_flat_compressed_hyperkzg_public_setup_from_reader(
        &std::fs::File::open("test_assets/ppot_0080_10.bin").unwrap(),
        ark_serialize::Validate::Yes,
    )
    .unwrap();

    ps.insert(0, ark_bn254::G1Affine::generator());

    (ps, vk)
}

#[cfg(all(test, feature = "std"))]
mod std_tests {
    use super::*;

    #[test]
    fn we_can_deserialize_empty_setup_from_slice() {
        assert_eq!(
            deserialize_flat_compressed_hyperkzg_public_setup_from_slice(&[], Validate::Yes)
                .unwrap(),
            Vec::<G1Affine>::new(),
        );
    }

    #[test]
    fn we_can_deserialize_empty_setup_from_reader() {
        let empty: &[u8] = &[];
        assert_eq!(
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader(empty, Validate::Yes)
                .unwrap(),
            Vec::<G1Affine>::new(),
        );
    }

    #[test]
    fn we_can_deserialize_ppot_02_setup_from_slice() {
        let bytes = include_bytes!("test_ppot_0080_02.bin");
        assert_eq!(
            deserialize_flat_compressed_hyperkzg_public_setup_from_slice(bytes, Validate::Yes)
                .unwrap()
                .len(),
            4,
        );
    }

    #[test]
    fn we_can_deserialize_ppot_02_setup_from_reader() {
        let file =
            std::fs::File::open("src/proof_primitive/hyperkzg/test_ppot_0080_02.bin").unwrap();
        assert_eq!(
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader(&file, Validate::Yes)
                .unwrap()
                .len(),
            4,
        );
    }
}
