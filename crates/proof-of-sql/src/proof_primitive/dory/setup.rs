use super::{G1Affine, G2Affine, PublicParameters, GT};
use crate::base::impl_serde_for_ark_serde_unchecked;
use alloc::vec::Vec;
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError, Valid};
use itertools::MultiUnzip;
use num_traits::One;
#[cfg(feature = "std")]
use std::{
    io::{Read, Write},
    sync::Arc,
};
/// The transparent setup information that the prover must know to create a proof.
/// This is public knowledge and must match with the verifier's setup information.
/// See Section 3.3 of <https://eprint.iacr.org/2020/1274.pdf> for details.
///
///
/// Note:
/// We use nu = m and k = m-i or m-j.
/// This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
pub struct ProverSetup<'a> {
    /// `Gamma_1[k]` = Γ_1,(m-k) in the Dory paper.
    pub(super) Gamma_1: Vec<&'a [G1Affine]>,
    /// `Gamma_2[k]` = Γ_2,(m-k) in the Dory paper.
    pub(super) Gamma_2: Vec<&'a [G2Affine]>,
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
    blitzar_handle:
        blitzar::compute::MsmHandle<blitzar::compute::ElementP2<ark_bls12_381::g1::Config>>,
}

impl<'a> ProverSetup<'a> {
    /// Create a new `ProverSetup` from the public parameters.
    /// # Panics
    /// Panics if the length of `Gamma_1` or `Gamma_2` is not equal to `2^max_nu`.
    pub(super) fn new(
        Gamma_1: &'a [G1Affine],
        Gamma_2: &'a [G2Affine],
        H_1: G1Affine,
        H_2: G2Affine,
        Gamma_2_fin: G2Affine,
        max_nu: usize,
    ) -> Self {
        assert_eq!(Gamma_1.len(), 1 << max_nu);
        assert_eq!(Gamma_2.len(), 1 << max_nu);
        #[cfg(feature = "blitzar")]
        let blitzar_handle = blitzar::compute::MsmHandle::new(
            &Gamma_1.iter().copied().map(Into::into).collect::<Vec<_>>(),
        );
        let (Gamma_1, Gamma_2): (Vec<_>, Vec<_>) = (0..=max_nu)
            .map(|k| (&Gamma_1[..1 << k], &Gamma_2[..1 << k]))
            .unzip();
        ProverSetup {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
            #[cfg(feature = "blitzar")]
            blitzar_handle,
        }
    }

    /// Function to save `ProverSetup` to a file in binary form
    #[cfg(feature = "std")]
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        // Serialize the ProverSetup struct into a buffer
        let mut serialized_data = Vec::new();
        self.serialize_with_mode(&mut serialized_data, ark_serialize::Compress::No)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))?;

        // Write serialized bytes to the file
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }

    /// Function to load `ProverSetup` from a file in binary form
    #[cfg(feature = "std")]
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        // Read the serialized data from the file
        let mut serialized_data = Vec::new();
        reader.read_to_end(&mut serialized_data)?;

        // Deserialize the data into a ProverSetup instance
        ProverSetup::deserialize_with_mode(
            &mut &serialized_data[..],
            ark_serialize::Compress::No,
            ark_serialize::Validate::Yes,
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "ProverSetup::blitzar_msm", level = "debug", skip_all)]
    pub(super) fn blitzar_msm(
        &self,
        res: &mut [blitzar::compute::ElementP2<ark_bls12_381::g1::Config>],
        element_num_bytes: u32,
        scalars: &[u8],
    ) {
        self.blitzar_handle.msm(res, element_num_bytes, scalars);
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "ProverSetup::blitzar_packed_msm", level = "debug", skip_all)]
    pub(super) fn blitzar_packed_msm(
        &self,
        res: &mut [blitzar::compute::ElementP2<ark_bls12_381::g1::Config>],
        output_bit_table: &[u32],
        scalars: &[u8],
    ) {
        self.blitzar_handle
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
        self.blitzar_handle
            .vlen_msm(res, output_bit_table, output_lengths, scalars);
    }
}

impl<'a> From<&'a PublicParameters> for ProverSetup<'a> {
    fn from(value: &'a PublicParameters) -> Self {
        Self::new(
            &value.Gamma_1,
            &value.Gamma_2,
            value.H_1,
            value.H_2,
            value.Gamma_2_fin,
            value.max_nu,
        )
    }
}

#[cfg(feature = "std")]
impl<'a> CanonicalSerialize for ProverSetup<'a> {
    fn serialize_with_mode<W: ark_serialize::Write>(
        &self,
        mut writer: W,
        compress: ark_serialize::Compress,
    ) -> Result<(), SerializationError> {
        // Serialize max_nu (usize as u64)
        (self.max_nu as u64).serialize_with_mode(&mut writer, compress)?;

        // Serialize Gamma_1 (Vec<&'a [G1Affine]>)
        for gamma_1_level in &self.Gamma_1 {
            gamma_1_level
                .iter()
                .try_for_each(|g1| g1.serialize_with_mode(&mut writer, compress))?;
        }

        // Serialize Gamma_2 (Vec<&'a [G2Affine]>)
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

        // TODO: serialize blitzar handle if necessary

        Ok(())
    }

    fn serialized_size(&self, compress: ark_serialize::Compress) -> usize {
        // Size of max_nu (u64 is 8 bytes)
        let max_nu_size = 8;

        // Size of Gamma_1 (Vec<&'a [G1Affine]>)
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

        // Size of Gamma_2 (Vec<&'a [G2Affine]>)
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

#[cfg(feature = "std")]
impl<'a> CanonicalDeserialize for ProverSetup<'a> {
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

        // Deserialize Gamma_1 as a flat Vec<G1Affine> and then convert it to Arc<[G1Affine]>
        let flat_gamma_1: Arc<[G1Affine]> = Arc::from(
            (0..total_elements)
                .map(|_| G1Affine::deserialize_with_mode(&mut reader, compress, validate))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(), // Convert Vec to Box<[T]>, then to Arc<[T]>
        );

        // Deserialize Gamma_2 as a flat Vec<G2Affine> and then convert it to Arc<[G2Affine]>
        let flat_gamma_2: Arc<[G2Affine]> = Arc::from(
            (0..total_elements)
                .map(|_| G2Affine::deserialize_with_mode(&mut reader, compress, validate))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(), // Convert Vec to Box<[T]>, then to Arc<[T]>
        );

        // Manually construct Gamma_1 as Vec<&'a [G1Affine]>
        let mut Gamma_1: Vec<&[G1Affine]> = Vec::with_capacity(max_nu + 1);
        let mut offset = 0;
        for k in 0..=max_nu {
            let level_size = 1 << k;
            let slice = &flat_gamma_1[offset..offset + level_size]; // Reference from Arc<[T]>
            Gamma_1.push(slice);
            offset += level_size;
        }

        // Manually construct Gamma_2 as Vec<&[G2Affine]>
        let mut Gamma_2: Vec<&[G2Affine]> = Vec::with_capacity(max_nu + 1);
        let mut offset = 0;
        for k in 0..=max_nu {
            let level_size = 1 << k;
            let slice = &flat_gamma_2[offset..offset + level_size]; // Reference from Arc<[T]>
            Gamma_2.push(slice);
            offset += level_size;
        }

        // Deserialize H_1 (G1Affine)
        let H_1 = G1Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize H_2 (G2Affine)
        let H_2 = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // Deserialize Gamma_2_fin (G2Affine)
        let Gamma_2_fin = G2Affine::deserialize_with_mode(&mut reader, compress, validate)?;

        // TODO: deserialize blitzar handle

        // Return the deserialized ProverSetup
        // #[allow(unreachable_code)]
        Ok(ProverSetup {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            Gamma_2_fin,
            max_nu,
            // #[cfg(feature = "blitzar")]
            blitzar_handle: todo!(),
        })
    }
}

#[cfg(feature = "std")]
impl<'a> Valid for ProverSetup<'a> {
    fn check(&self) -> Result<(), SerializationError> {
        // Check all slices of G1Affine in Gamma_1
        for gamma_1_level in &self.Gamma_1 {
            gamma_1_level
                .iter()
                .try_for_each(ark_serialize::Valid::check)?;
        }

        // Check all slices of G2Affine in Gamma_2
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

/// The transparent setup information that the verifier must know to verify a proof.
/// This is public knowledge and must match with the prover's setup information.
/// See Section 3.3 of <https://eprint.iacr.org/2020/1274.pdf> for details.
///
///
/// Note:
/// We use nu = m and k = m-i or m-j.
/// This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
#[derive(CanonicalSerialize, CanonicalDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct VerifierSetup {
    /// `Delta_1L[k]` = Δ_1L,(m-k) in the Dory paper, so `Delta_1L[0]` is unused. Note, this is the same as `Delta_2L`.
    pub(super) Delta_1L: Vec<GT>,
    /// `Delta_1R[k]` = Δ_1R,(m-k) in the Dory paper, so `Delta_1R[0]` is unused.
    pub(super) Delta_1R: Vec<GT>,
    /// `Delta_2L[k]` = Δ_2L,(m-k) in the Dory paper, so `Delta_2L[0]` is unused. Note, this is the same as `Delta_1L`.
    pub(super) Delta_2L: Vec<GT>,
    /// `Delta_2R[k]` = Δ_2R,(m-k) in the Dory paper, so `Delta_2R[0]` is unused.
    pub(super) Delta_2R: Vec<GT>,
    /// `chi[k]` = χ,(m-k) in the Dory paper.
    pub(super) chi: Vec<GT>,
    /// `Gamma_1_0` is the `Γ_1` used in Scalar-Product algorithm in the Dory paper.
    pub(super) Gamma_1_0: G1Affine,
    /// `Gamma_2_0` is the `Γ_2` used in Scalar-Product algorithm in the Dory paper.
    pub(super) Gamma_2_0: G2Affine,
    /// `H_1` = `H_1` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1Affine,
    /// `H_2` = `H_2` in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2Affine,
    /// `H_T` = `H_T` in the Dory paper.
    pub(super) H_T: GT,
    /// `Gamma_2_fin` = `Gamma_2,fin` in the Dory paper.
    pub(super) Gamma_2_fin: G2Affine,
    /// `max_nu` is the maximum nu that this setup will work for
    pub(super) max_nu: usize,
}

impl_serde_for_ark_serde_unchecked!(VerifierSetup);

impl VerifierSetup {
    /// Create a new `VerifierSetup` from the public parameters.
    /// # Panics
    /// Panics if the length of `Gamma_1_nu` is not equal to `2^max_nu`.
    /// Panics if the length of `Gamma_2_nu` is not equal to `2^max_nu`.
    pub(super) fn new(
        Gamma_1_nu: &[G1Affine],
        Gamma_2_nu: &[G2Affine],
        H_1: G1Affine,
        H_2: G2Affine,
        Gamma_2_fin: G2Affine,
        max_nu: usize,
    ) -> Self {
        assert_eq!(Gamma_1_nu.len(), 1 << max_nu);
        assert_eq!(Gamma_2_nu.len(), 1 << max_nu);
        let (Delta_1L_2L, Delta_1R, Delta_2R, chi): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = (0..=max_nu)
            .map(|k| {
                if k == 0 {
                    (
                        PairingOutput(One::one()),
                        PairingOutput(One::one()),
                        PairingOutput(One::one()),
                        Pairing::pairing(Gamma_1_nu[0], Gamma_2_nu[0]),
                    )
                } else {
                    (
                        Pairing::multi_pairing(
                            &Gamma_1_nu[..1 << (k - 1)],
                            &Gamma_2_nu[..1 << (k - 1)],
                        ),
                        Pairing::multi_pairing(
                            &Gamma_1_nu[1 << (k - 1)..1 << k],
                            &Gamma_2_nu[..1 << (k - 1)],
                        ),
                        Pairing::multi_pairing(
                            &Gamma_1_nu[..1 << (k - 1)],
                            &Gamma_2_nu[1 << (k - 1)..1 << k],
                        ),
                        Pairing::multi_pairing(&Gamma_1_nu[..1 << k], &Gamma_2_nu[..1 << k]),
                    )
                }
            })
            .multiunzip();
        Self {
            Delta_1L: Delta_1L_2L.clone(),
            Delta_1R,
            Delta_2L: Delta_1L_2L,
            Delta_2R,
            chi,
            Gamma_1_0: Gamma_1_nu[0],
            Gamma_2_0: Gamma_2_nu[0],
            H_1,
            H_2,
            H_T: Pairing::pairing(H_1, H_2),
            Gamma_2_fin,
            max_nu,
        }
    }
}

impl From<&PublicParameters> for VerifierSetup {
    fn from(value: &PublicParameters) -> Self {
        Self::new(
            &value.Gamma_1,
            &value.Gamma_2,
            value.H_1,
            value.H_2,
            value.Gamma_2_fin,
            value.max_nu,
        )
    }
}
