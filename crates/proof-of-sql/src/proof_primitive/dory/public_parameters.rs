use super::{G1Affine, G2Affine};
use alloc::vec::Vec;
/// The public parameters for the Dory protocol. See section 5 of https://eprint.iacr.org/2020/1274.pdf for details.
///
/// Note: even though H_1 and H_2 are marked as blue, they are still needed.
///
/// Note: Gamma_1_fin is unused, so we leave it out.
pub struct PublicParameters {
    /// This is the vector of G1 elements that are used in the Dory protocol. That is, Γ_1,0 in the Dory paper.
    pub(super) Gamma_1: Vec<G1Affine>,
    /// This is the vector of G2 elements that are used in the Dory protocol. That is, Γ_2,0 in the Dory paper.
    pub(super) Gamma_2: Vec<G2Affine>,
    /// `H_1` = H_1 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_1: G1Affine,
    /// `H_2` = H_2 in the Dory paper. This could be used for blinding, but is currently only used in the Fold-Scalars algorithm.
    pub(super) H_2: G2Affine,
    /// `Gamma_2_fin` = Gamma_2,fin in the Dory paper.
    pub(super) Gamma_2_fin: G2Affine,
    /// `max_nu` is the maximum nu that this setup will work for.
    pub(super) max_nu: usize,
}

impl PublicParameters {
    #[cfg(any(test, feature = "test"))]
    /// Generate random public parameters for testing purposes.
    pub fn rand<R>(max_nu: usize, rng: &mut R) -> Self
    where
        R: ark_std::rand::Rng + ?Sized,
    {
        use ark_std::UniformRand;
        let (Gamma_1, Gamma_2) = super::rand_G_vecs(max_nu, rng);
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
}
