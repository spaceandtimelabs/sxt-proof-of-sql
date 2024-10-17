use super::{G1Affine, G2Affine, PublicParameters};
use alloc::vec::Vec;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError, Valid};
#[cfg(feature = "std")]
use ark_serialize::{Compress, Validate};
#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Write},
    path::Path,
};
/// The transparent setup information that the prover must know to create a proof.
/// This is public knowledge and must match with the verifier's setup information.
/// See Section 3.3 of <https://eprint.iacr.org/2020/1274.pdf> for details.
///
/// This duplicates the [`ProverSetup`] type because file I/O needs owned data.
/// This approach is the least disruptive change to the rest of library which
/// relied on borrown prover data.
///
/// Note:
/// We use nu = m and k = m-i or m-j.
/// This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
pub struct OwnedProverSetup {
    /// `Gamma_1[k]` = Γ_1,(m-k) in the Dory paper.
    pub(super) Gamma_1: Vec<Vec<G1Affine>>,
    /// `Gamma_2[k]` = Γ_2,(m-k) in the Dory paper.
    pub(super) Gamma_2: Vec<Vec<G2Affine>>,
    /// `H_1` = `H_1` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1Affine,
    /// `H_2` = `H_2` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2Affine,
    /// `Gamma_2_fin` = `Gamma_2,fin` in the Dory paper.
    pub(super) Gamma_2_fin: G2Affine,
    /// `max_nu` is the maximum nu that this setup will work for
    pub(super) max_nu: usize,
    /// The handle to the `blitzar` `Gamma_1` instances.
    #[cfg(feature = "blitzar")]
    _blitzar_handle:
        blitzar::compute::MsmHandle<blitzar::compute::ElementP2<ark_bls12_381::g1::Config>>,
}

impl OwnedProverSetup {
    /// Create a new `OwnedProverSetup` from the public parameters.
    /// # Panics
    /// Panics if the length of `Gamma_1` or `Gamma_2` is not equal to `2^max_nu`.
    pub(super) fn new(
        Gamma_1: Vec<G1Affine>,
        Gamma_2: Vec<G2Affine>,
        H_1: G1Affine,
        H_2: G2Affine,
        Gamma_2_fin: G2Affine,
        max_nu: usize,
    ) -> Self {
        assert_eq!(Gamma_1.len(), 1 << max_nu);
        assert_eq!(Gamma_2.len(), 1 << max_nu);

        #[cfg(feature = "blitzar")]
        let _blitzar_handle = blitzar::compute::MsmHandle::new(
            &Gamma_1.iter().copied().map(Into::into).collect::<Vec<_>>(),
        );

        // Convert slices to Vecs of owned elements
        let (Gamma_1, Gamma_2): (Vec<Vec<G1Affine>>, Vec<Vec<G2Affine>>) = (0..=max_nu)
            .map(|k| {
                (
                    Gamma_1[..1 << k].to_vec(), // Clone the slice into a Vec<G1Affine>
                    Gamma_2[..1 << k].to_vec(), // Clone the slice into a Vec<G2Affine>
                )
            })
            .unzip();

        OwnedProverSetup {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
            #[cfg(feature = "blitzar")]
            _blitzar_handle,
        }
    }

    #[cfg(feature = "std")]
    /// Function to save `OwnedProverSetup` to a file in binary form
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Serialize the OwnedProverSetup struct into a buffer
        let mut serialized_data = Vec::new();
        self.serialize_with_mode(&mut serialized_data, Compress::No)
            .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))?;

        // Write serialized bytes to the file
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }

    #[cfg(feature = "std")]
    /// Function to load `OwnedProverSetup` from a file in binary form
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read the serialized data from the file
        let mut serialized_data = Vec::new();
        reader.read_to_end(&mut serialized_data)?;

        // Deserialize the data into an OwnedProverSetup instance
        OwnedProverSetup::deserialize_with_mode(
            &mut &serialized_data[..],
            Compress::No,
            Validate::Yes,
        )
        .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "ProverSetup::blitzar_msm", level = "debug", skip_all)]
    pub(super) fn blitzar_msm(
        &self,
        res: &mut [blitzar::compute::ElementP2<ark_bls12_381::g1::Config>],
        element_num_bytes: u32,
        scalars: &[u8],
    ) {
        self._blitzar_handle.msm(res, element_num_bytes, scalars);
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "ProverSetup::blitzar_packed_msm", level = "debug", skip_all)]
    pub(super) fn blitzar_packed_msm(
        &self,
        res: &mut [blitzar::compute::ElementP2<ark_bls12_381::g1::Config>],
        output_bit_table: &[u32],
        scalars: &[u8],
    ) {
        self._blitzar_handle
            .packed_msm(res, output_bit_table, scalars);
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "ProverSetup::blitzar_vlen_msm", level = "debug", skip_all)]
    pub(super) fn blitzar_vlen_msm(
        &self,
        res: &mut [blitzar::compute::ElementP2<ark_bls12_381::g1::Config>],
        output_bit_table: &[u32],
        output_lengths: &[u32],
        scalars: &[u8],
    ) {
        self._blitzar_handle
            .vlen_msm(res, output_bit_table, output_lengths, scalars);
    }
}

impl From<PublicParameters> for OwnedProverSetup {
    fn from(value: PublicParameters) -> Self {
        Self::new(
            value.Gamma_1,
            value.Gamma_2,
            value.H_1,
            value.H_2,
            value.Gamma_2_fin,
            value.max_nu,
        )
    }
}

impl CanonicalSerialize for OwnedProverSetup {
    fn serialize_with_mode<W: ark_serialize::Write>(
        &self,
        mut writer: W,
        compress: ark_serialize::Compress,
    ) -> Result<(), SerializationError> {
        // Serialize max_nu (usize as u64)
        (self.max_nu as u64).serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_1 (Vec<Vec<G1Affine>>)
        for gamma_1_level in &self.Gamma_1 {
            gamma_1_level
                .iter()
                .try_for_each(|g1| g1.serialize_with_mode(&mut writer, compress))?;
        }

        // Serialize Gamma_2 (Vec<Vec<G2Affine>>)
        for gamma_2_level in &self.Gamma_2 {
            gamma_2_level
                .iter()
                .try_for_each(|g2| g2.serialize_with_mode(&mut writer, compress))?;
        }

        // Serialize H_1 (G1Affine)
        self.H_1.serialize_with_mode(&mut writer, compress)?;

        // Serialize H_2 (G2Affine)
        self.H_2.serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_2_fin (G2Affine)
        self.Gamma_2_fin
            .serialize_with_mode(&mut writer, compress)?;

        // TODO: serialize blitzar handle

        Ok(())
    }

    // Update serialized_size
    fn serialized_size(&self, compress: ark_serialize::Compress) -> usize {
        // Size of max_nu (u64 is 8 bytes)
        let max_nu_size = 8;

        // Size of Gamma_1 (Vec<Vec<G1Affine>>)
        let gamma_1_size: usize = self
            .Gamma_1
            .iter()
            .map(|gamma_1_level| {
                gamma_1_level
                    .iter()
                    .map(|g1| g1.serialized_size(compress))
                    .sum::<usize>()
            })
            .sum();

        // Size of Gamma_2 (Vec<Vec<G2Affine>>)
        let gamma_2_size: usize = self
            .Gamma_2
            .iter()
            .map(|gamma_2_level| {
                gamma_2_level
                    .iter()
                    .map(|g2| g2.serialized_size(compress))
                    .sum::<usize>()
            })
            .sum();

        // Size of H_1 (G1Affine)
        let h1_size = self.H_1.serialized_size(compress);

        // Size of H_2 (G2Affine)
        let h2_size = self.H_2.serialized_size(compress);

        // Size of Gamma_2_fin (G2Affine)
        let gamma_2_fin_size = self.Gamma_2_fin.serialized_size(compress);

        // Sum everything to get the total size
        max_nu_size + gamma_1_size + gamma_2_size + h1_size + h2_size + gamma_2_fin_size

        // TODO: include size of blitzar handle
    }
}

impl CanonicalDeserialize for OwnedProverSetup {
    fn deserialize_with_mode<R: ark_serialize::Read>(
        mut reader: R,
        compress: ark_serialize::Compress,
        validate: ark_serialize::Validate,
    ) -> Result<Self, SerializationError> {
        // Deserialize max_nu (u64)
        let max_nu_u64 = u64::deserialize_with_mode(&mut reader, compress, validate)?;
        let max_nu = max_nu_u64 as usize;

        // Calculate the total number of elements to be deserialized (2^max_nu)
        let total_elements = 1 << max_nu;

        // Deserialize Gamma_1 as a flat Vec<G1Affine>
        let flat_gamma_1: Vec<G1Affine> = (0..total_elements)
            .map(|_| G1Affine::deserialize_with_mode(&mut reader, compress, validate))
            .collect::<Result<_, _>>()?;

        // Deserialize Gamma_2 as a flat Vec<G2Affine>
        let flat_gamma_2: Vec<G2Affine> = (0..total_elements)
            .map(|_| G2Affine::deserialize_with_mode(&mut reader, compress, validate))
            .collect::<Result<_, _>>()?;

        // Convert flat Gamma_1 and Gamma_2 into nested Vecs (Vec<Vec<G1Affine>> and Vec<Vec<G2Affine>>)
        let Gamma_1: Vec<Vec<G1Affine>> = (0..=max_nu)
            .map(|k| flat_gamma_1[..1 << k].to_vec()) // Create nested Vec<G1Affine> for each level
            .collect();

        let Gamma_2: Vec<Vec<G2Affine>> = (0..=max_nu)
            .map(|k| flat_gamma_2[..1 << k].to_vec()) // Create nested Vec<G2Affine> for each level
            .collect();

        // Deserialize H_1 (G1Affine)
        let H_1 = G1Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize H_2 (G2Affine)
        let H_2 = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize Gamma_2_fin (G2Affine)
        let Gamma_2_fin = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // TODO: deserialize blitzar handle

        // Return the deserialized OwnedProverSetup
        #[allow(unreachable_code)]
        Ok(Self {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
            #[cfg(feature = "blitzar")]
            _blitzar_handle: todo!(), // Handle this separately if necessary
        })
    }
}

impl Valid for OwnedProverSetup {
    fn check(&self) -> Result<(), SerializationError> {
        // Check all nested vectors of G1Affine in Gamma_1
        for gamma_1_level in &self.Gamma_1 {
            gamma_1_level
                .iter()
                .try_for_each(ark_serialize::Valid::check)?;
        }

        // Check all nested vectors of G2Affine in Gamma_2
        for gamma_2_level in &self.Gamma_2 {
            gamma_2_level
                .iter()
                .try_for_each(ark_serialize::Valid::check)?;
        }

        // Check individual G1Affine H_1
        self.H_1.check()?;

        // Check individual G2Affine H_2
        self.H_2.check()?;

        // Check individual G2Affine Gamma_2_fin
        self.Gamma_2_fin.check()?;

        // TODO: check blitzar handle

        Ok(())
    }
}
