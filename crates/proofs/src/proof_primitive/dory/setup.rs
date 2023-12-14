use super::{PublicParameters, G1, G2, GT};
use ark_ec::pairing::Pairing;
use itertools::MultiUnzip;

/// The transparent setup information that the prover must know to create a proof.
/// This is public knowledge and must match with the verifier's setup information.
/// See Section 3.3 of https://eprint.iacr.org/2020/1274.pdf for details.
///
///
/// Note:
/// We use nu = m and k = m-i or m-j.
/// This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
pub struct ProverSetup<'a> {
    /// `Gamma_1[k]` = Γ_1,(m-k) in the Dory paper.
    pub(super) Gamma_1: Vec<&'a [G1]>,
    /// `Gamma_2[k]` = Γ_2,(m-k) in the Dory paper.
    pub(super) Gamma_2: Vec<&'a [G2]>,
    /// `H_1` = H_1 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1,
    /// `H_2` = H_2 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2,
    /// `max_nu` is the maximum nu that this setup will work for
    pub(super) max_nu: usize,
}

impl<'a> ProverSetup<'a> {
    /// Create a new `ProverSetup` from the public parameters.
    pub(super) fn new(
        Gamma_1: &'a [G1],
        Gamma_2: &'a [G2],
        H_1: G1,
        H_2: G2,
        max_nu: usize,
    ) -> Self {
        assert_eq!(Gamma_1.len(), 1 << max_nu);
        assert_eq!(Gamma_2.len(), 1 << max_nu);
        let (Gamma_1, Gamma_2) = (0..max_nu + 1)
            .map(|k| (&Gamma_1[..1 << k], &Gamma_2[..1 << k]))
            .unzip();
        ProverSetup {
            Gamma_1,
            Gamma_2,
            H_1,
            H_2,
            max_nu,
        }
    }
}

impl<'a> From<&'a PublicParameters> for ProverSetup<'a> {
    fn from(value: &'a PublicParameters) -> Self {
        Self::new(
            &value.Gamma_1,
            &value.Gamma_2,
            value.H_1,
            value.H_2,
            value.max_nu,
        )
    }
}

/// The transparent setup information that the verifier must know to verify a proof.
/// This is public knowledge and must match with the prover's setup information.
/// See Section 3.3 of https://eprint.iacr.org/2020/1274.pdf for details.
///
///
/// Note:
/// We use nu = m and k = m-i or m-j.
/// This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
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
    /// `Gamma_1_0` is the Γ_1 used in Scalar-Product algorithm in the Dory paper.
    pub(super) Gamma_1_0: G1,
    /// `Gamma_2_0` is the Γ_2 used in Scalar-Product algorithm in the Dory paper.
    pub(super) Gamma_2_0: G2,
    /// `H_1` = H_1 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1,
    /// `H_2` = H_2 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2,
    /// `H_T` = H_T in the Dory paper.
    pub(super) H_T: GT,
    /// `max_nu` is the maximum nu that this setup will work for
    pub(super) max_nu: usize,
}

impl VerifierSetup {
    /// Create a new `VerifierSetup` from the public parameters.
    pub(super) fn new(
        Gamma_1_nu: &[G1],
        Gamma_2_nu: &[G2],
        H_1: G1,
        H_2: G2,
        max_nu: usize,
    ) -> Self {
        assert_eq!(Gamma_1_nu.len(), 1 << max_nu);
        assert_eq!(Gamma_2_nu.len(), 1 << max_nu);
        let (Delta_1L_2L, Delta_1R, Delta_2R, chi): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = (0..max_nu
            + 1)
            .map(|k| {
                if k == 0 {
                    (
                        Default::default(),
                        Default::default(),
                        Default::default(),
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
            value.max_nu,
        )
    }
}
