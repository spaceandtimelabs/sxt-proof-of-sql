use super::{G1Affine, G2Affine};
use alloc::vec::Vec;
use ark_ff::UniformRand;
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use ark_std::rand::{CryptoRng, Rng};
use core::iter;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

/// The public parameters for the Dory protocol. See section 5 of <https://eprint.iacr.org/2020/1274.pdf> for details.
///
/// Note: even though `H_1` and `H_2` are marked as blue, they are still needed.
///
/// Note: `Gamma_1_fin` is unused, so we leave it out.
pub struct PublicParameters {
    /// This is the vector of G1 elements that are used in the Dory protocol. That is, `Γ_1,0` in the Dory paper.
    pub(super) Gamma_1: Vec<G1Affine>,
    /// This is the vector of G2 elements that are used in the Dory protocol. That is, `Γ_2,0` in the Dory paper.
    pub(super) Gamma_2: Vec<G2Affine>,
    /// `H_1` = `H_1` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1Affine,
    /// `H_2` = `H_2` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2Affine,
    /// `Gamma_2_fin` = `Gamma_2,fin` in the Dory paper.
    pub(super) Gamma_2_fin: G2Affine,
    /// `max_nu` is the maximum nu that this setup will work for.
    pub(super) max_nu: usize,
}

impl PublicParameters {
    /// Generate cryptographically secure random public parameters.
    pub fn rand<R: CryptoRng + Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        Self::rand_impl(max_nu, rng)
    }
    #[cfg(any(test, feature = "test"))]
    /// Generate random public parameters for testing.
    pub fn test_rand<R: Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        Self::rand_impl(max_nu, rng)
    }
    fn rand_impl<R: Rng + ?Sized>(max_nu: usize, rng: &mut R) -> Self {
        let (Gamma_1, Gamma_2) = iter::repeat_with(|| (G1Affine::rand(rng), G2Affine::rand(rng)))
            .take(1 << max_nu)
            .unzip();
        let (H_1, H_2) = (G1Affine::rand(rng), G2Affine::rand(rng));
        let Gamma_2_fin = G2Affine::rand(rng);

        Self {
            Gamma_1,
            Gamma_2,
            max_nu,
            H_1,
            H_2,
            Gamma_2_fin,
        }
    }

    /// Function to save PublicParameters to a file in binary form
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Create or open the file at the specified path
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Serialize the PublicParameters struct into the file
        let mut serialized_data = Vec::new();
        self.serialize_with_mode(&mut serialized_data, Compress::No)
            .expect("Failed to serialize PublicParameters");

        // Write serialized bytes to the file
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }

    /// Function to load PublicParameters from a file in binary form
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        // Open the file at the specified path
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read the serialized data from the file
        let mut serialized_data = Vec::new();
        reader.read_to_end(&mut serialized_data)?;

        // Deserialize the data into a PublicParameters instance
        let params = PublicParameters::deserialize_with_mode(
            &mut &serialized_data[..],
            Compress::No,
            Validate::Yes,
        )
        .expect("Failed to deserialize PublicParameters");

        Ok(params)
    }
}

impl CanonicalSerialize for PublicParameters {
    fn serialize_with_mode<W: std::io::Write>(
        &self,
        mut writer: W,
        compress: ark_serialize::Compress,
    ) -> Result<(), SerializationError> {
        // Serialize max_nu (usize as u64) first
        let max_nu_u64 = self.max_nu as u64;
        max_nu_u64.serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_1 (Vec<G1Affine>)
        for g1 in &self.Gamma_1 {
            g1.serialize_with_mode(&mut writer, compress)?;
        }

        // Serialize Gamma_2 (Vec<G2Affine>)
        for g2 in &self.Gamma_2 {
            g2.serialize_with_mode(&mut writer, compress)?;
        }

        // Serialize H_1 (G1Affine)
        self.H_1.serialize_with_mode(&mut writer, compress)?;

        // Serialize H_2 (G2Affine)
        self.H_2.serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_2_fin (G2Affine)
        self.Gamma_2_fin
            .serialize_with_mode(&mut writer, compress)?;

        Ok(())
    }

    // Update serialized_size accordingly
    fn serialized_size(&self, compress: ark_serialize::Compress) -> usize {
        let mut size = 0;

        // Size of max_nu
        size += 8; // u64 is 8 bytes

        // Calculate size of Gamma_1 (Vec<G1Affine>)
        for g1 in &self.Gamma_1 {
            size += g1.serialized_size(compress);
        }

        // Calculate size of Gamma_2 (Vec<G2Affine>)
        for g2 in &self.Gamma_2 {
            size += g2.serialized_size(compress);
        }

        // Calculate size of H_1 (G1Affine)
        size += self.H_1.serialized_size(compress);

        // Calculate size of H_2 (G2Affine)
        size += self.H_2.serialized_size(compress);

        // Calculate size of Gamma_2_fin (G2Affine)
        size += self.Gamma_2_fin.serialized_size(compress);

        size
    }
}

impl CanonicalDeserialize for PublicParameters {
    fn deserialize_with_mode<R: std::io::Read>(
        mut reader: R,
        compress: ark_serialize::Compress,
        validate: ark_serialize::Validate,
    ) -> Result<Self, SerializationError> {
        // Deserialize max_nu (u64)
        let max_nu_u64 = u64::deserialize_with_mode(&mut reader, compress, validate)?;
        let max_nu = max_nu_u64 as usize;

        // Deserialize Gamma_1 (Vec<G1Affine>)
        let mut Gamma_1 = Vec::with_capacity(1 << max_nu);
        for _ in 0..(1 << max_nu) {
            Gamma_1.push(G1Affine::deserialize_with_mode(
                &mut reader,
                compress,
                validate,
            )?);
        }

        // Deserialize Gamma_2 (Vec<G2Affine>)
        let mut Gamma_2 = Vec::with_capacity(1 << max_nu);
        for _ in 0..(1 << max_nu) {
            Gamma_2.push(G2Affine::deserialize_with_mode(
                &mut reader,
                compress,
                validate,
            )?);
        }

        // Deserialize H_1 (G1Affine)
        let H_1 = G1Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize H_2 (G2Affine)
        let H_2 = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize Gamma_2_fin (G2Affine)
        let Gamma_2_fin = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        Ok(Self {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
        })
    }

    // Remove unnecessary methods if they're not overridden
}

// Implement the Valid trait to perform validation on deserialized data
impl Valid for PublicParameters {
    fn check(&self) -> Result<(), SerializationError> {
        // Check that all G1Affine and G2Affine elements are valid
        for g1 in &self.Gamma_1 {
            g1.check()?;
        }
        for g2 in &self.Gamma_2 {
            g2.check()?;
        }
        self.H_1.check()?;
        self.H_2.check()?;
        self.Gamma_2_fin.check()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_std::rand::thread_rng;
    use std::io::Cursor;

    #[test]
    fn we_can_serialize_and_deserialize_round_trip() {
        // Create a random PublicParameters instance
        let mut rng = thread_rng();
        let original_params = PublicParameters::rand(2, &mut rng);

        // Serialize the original parameters to a byte buffer
        let mut serialized_data = Vec::new();
        original_params
            .serialize_with_mode(&mut serialized_data, ark_serialize::Compress::No)
            .expect("Failed to serialize PublicParameters");

        // Deserialize the byte buffer back into a PublicParameters instance
        let mut reader = Cursor::new(serialized_data);
        let deserialized_params = PublicParameters::deserialize_with_mode(
            &mut reader,
            ark_serialize::Compress::No,
            ark_serialize::Validate::Yes,
        )
        .expect("Failed to deserialize PublicParameters");

        // Check that the original and deserialized parameters are the same
        assert_eq!(original_params.Gamma_1, deserialized_params.Gamma_1);
        assert_eq!(original_params.Gamma_2, deserialized_params.Gamma_2);
        assert_eq!(original_params.H_1, deserialized_params.H_1);
        assert_eq!(original_params.H_2, deserialized_params.H_2);
        assert_eq!(original_params.Gamma_2_fin, deserialized_params.Gamma_2_fin);
        assert_eq!(original_params.max_nu, deserialized_params.max_nu);

        // Validate the deserialized parameters to ensure correctness
        deserialized_params
            .check()
            .expect("Deserialized parameters are not valid");
    }

    // Observed proof size vs nu:
    // nu = 4  |  0.005 MB  | ▏
    // nu = 10 |  0.282 MB  | ███▏
    // nu = 12 |  1.125 MB  | █████████
    // nu = 14 |  4.500 MB  | ████████████████████████████
    // nu = 15 |  9.000 MB  | ██████████████████████████████████████████████
    #[test]
    fn we_can_read_and_write_a_file_round_trip() {
        let nu_values = vec![18];

        // Loop through each nu value
        for &nu in &nu_values {
            dbg!("\nTesting with nu = {}", nu);

            let start_time = std::time::Instant::now();

            // Create a random PublicParameters instance with the current nu value
            let mut rng = thread_rng();
            let original_params = PublicParameters::rand(nu, &mut rng);

            // File path in the current working directory
            let file_name = format!("public_params_{}.bin", nu);
            let file_path = Path::new(&file_name);

            original_params
                .save_to_file(file_path)
                .expect("Failed to save PublicParameters to file");

            // Load the PublicParameters from the file
            let loaded_params = PublicParameters::load_from_file(file_path)
                .expect("Failed to load PublicParameters from file");

            // Check that the original and loaded parameters are identical
            assert_eq!(original_params.Gamma_1, loaded_params.Gamma_1);
            assert_eq!(original_params.Gamma_2, loaded_params.Gamma_2);
            assert_eq!(original_params.H_1, loaded_params.H_1);
            assert_eq!(original_params.H_2, loaded_params.H_2);
            assert_eq!(original_params.Gamma_2_fin, loaded_params.Gamma_2_fin);
            assert_eq!(original_params.max_nu, loaded_params.max_nu);

            // Record the file size in bytes
            let metadata = std::fs::metadata(file_path).expect("Failed to get file metadata");
            let file_size = metadata.len(); // Get the file size in bytes
            dbg!("File size for nu = {}: {} bytes", nu, file_size);

            // Record the time taken and print it
            let elapsed_time = start_time.elapsed();
            dbg!("Time taken for nu = {}: {:?}", nu, elapsed_time);

            // Clean up the test file after the test runs
            std::fs::remove_file(file_path).expect("Failed to remove test file");
        }
    }
}
