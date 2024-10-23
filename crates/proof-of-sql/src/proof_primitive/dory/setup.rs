use super::{G1Affine, G2Affine, PublicParameters, GT};
use crate::base::impl_serde_for_ark_serde_unchecked;
use alloc::vec::Vec;
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use itertools::MultiUnzip;
use num_traits::One;
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
    pub blitzar_handle:
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

    /// Create a new `ProverSetup` from the public parameters and blitzar handle
    /// # Panics
    /// Panics if the length of `Gamma_1` or `Gamma_2` is not equal to `2^max_nu`.
    #[must_use]
    #[cfg(feature = "blitzar")]
    pub fn from_public_parameters_and_blitzar_handle(
        public_parameters: &'a PublicParameters,
        blitzar_handle: blitzar::compute::MsmHandle<
            blitzar::compute::ElementP2<ark_bls12_381::g1::Config>,
        >,
    ) -> Self {
        let Gamma_1: &'a [G1Affine] = &public_parameters.Gamma_1;
        let Gamma_2: &'a [G2Affine] = &public_parameters.Gamma_2;
        let H_1 = public_parameters.H_1;
        let H_2 = public_parameters.H_2;
        let Gamma_2_fin = public_parameters.Gamma_2_fin;
        let max_nu = public_parameters.max_nu;
        assert_eq!(Gamma_1.len(), 1 << max_nu);
        assert_eq!(Gamma_2.len(), 1 << max_nu);

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

    /// Gets the `MSMHandle` for this setup
    #[must_use]
    #[cfg(feature = "blitzar")]
    pub fn blitzar_handle(
        self,
    ) -> blitzar::compute::MsmHandle<blitzar::compute::ElementP2<ark_bls12_381::g1::Config>> {
        self.blitzar_handle
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

    #[cfg(feature = "std")]
    /// Function to save `VerifierSetup` to a file in binary form
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Create or open the file at the specified path

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Serialize the PublicParameters struct into the file
        let mut serialized_data = Vec::new();
        self.serialize_with_mode(&mut serialized_data, Compress::No)
            .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))?;

        // Write serialized bytes to the file
        writer.write_all(&serialized_data)?;
        writer.flush()?;
        Ok(())
    }

    #[cfg(feature = "std")]
    /// Function to load `VerifierSetup` from a file in binary form
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        // Open the file at the specified path

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read the serialized data from the file
        let mut serialized_data = Vec::new();
        reader.read_to_end(&mut serialized_data)?;

        // Deserialize the data into a PublicParameters instance
        Self::deserialize_with_mode(&mut &serialized_data[..], Compress::No, Validate::No)
            .map_err(|e| Error::new(ErrorKind::Other, format!("{e}")))
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
