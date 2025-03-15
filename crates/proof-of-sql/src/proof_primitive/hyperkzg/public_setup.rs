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
