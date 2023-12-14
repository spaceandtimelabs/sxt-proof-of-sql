use super::{G1, G2};
/// The public parameters for the Dory protocol. See section 5 of https://eprint.iacr.org/2020/1274.pdf for details.
pub struct PublicParameters {
    pub(super) Gamma_1: Vec<G1>,
    pub(super) Gamma_2: Vec<G2>,
    pub(super) H_1: G1,
    pub(super) H_2: G2,
    pub(super) max_nu: usize,
}

impl PublicParameters {
    #[cfg(test)]
    pub fn rand<R>(max_nu: usize, rng: &mut R) -> Self
    where
        R: ark_std::rand::Rng + ?Sized,
    {
        use ark_std::UniformRand;
        let (Gamma_1, Gamma_2) = super::rand_G_vecs(max_nu, rng);
        let (H_1, H_2) = (G1::rand(rng), G2::rand(rng));
        Self {
            Gamma_1,
            Gamma_2,
            max_nu,
            H_1,
            H_2,
        }
    }
}
