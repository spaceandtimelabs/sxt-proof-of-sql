#[cfg(test)]
use super::F;
use super::{G1, G2};
#[cfg(test)]
use ark_std::rand::{rngs::StdRng, Rng, SeedableRng};
use ark_std::UniformRand;

/// Create a random number generator for testing.
#[cfg(test)]
pub fn test_rng() -> impl Rng {
    ark_std::test_rng()
}

/// Create a random number generator for testing with a specific seed.
#[cfg(test)]
pub fn test_seed_rng(seed: [u8; 32]) -> impl Rng {
    StdRng::from_seed(seed)
}

/// Creates two vectors of random G1 and G2 elements with length 2^nu.
pub fn rand_G_vecs<R>(nu: usize, rng: &mut R) -> (Vec<G1>, Vec<G2>)
where
    R: ark_std::rand::Rng + ?Sized,
{
    core::iter::repeat_with(|| (G1::rand(rng), G2::rand(rng)))
        .take(1 << nu)
        .unzip()
}

/// Creates two vectors of random F elements with length 2^nu.
#[cfg(test)]
pub fn rand_F_vecs<R>(nu: usize, rng: &mut R) -> (Vec<F>, Vec<F>)
where
    R: ark_std::rand::Rng + ?Sized,
{
    core::iter::repeat_with(|| (F::rand(rng), F::rand(rng)))
        .take(1 << nu)
        .unzip()
}

#[test]
fn we_can_create_rand_G_vecs() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        assert_eq!(Gamma_1.len(), 1 << nu);
        assert_eq!(Gamma_2.len(), 1 << nu);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_consecutively_from_the_same_rng() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng);
        assert_ne!(Gamma_1, Gamma_1_2);
        assert_ne!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_the_same_rand_G_vecs_from_the_same_seed() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([1; 32]);
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng_2);
        assert_eq!(Gamma_1, Gamma_1_2);
        assert_eq!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_from_different_seeds() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([2; 32]);
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng_2);
        assert_ne!(Gamma_1, Gamma_1_2);
        assert_ne!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_rand_F_vecs() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        assert_eq!(s1.len(), 1 << nu);
        assert_eq!(s2.len(), 1 << nu);
        assert_ne!(s1, s2);
    }
}

#[test]
fn we_can_create_different_rand_F_vecs_consecutively_from_the_same_rng() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng);
        assert_ne!(s1, s1_2);
        assert_ne!(s2, s2_2);
    }
}

#[test]
fn we_can_create_the_same_rand_F_vecs_from_the_same_seed() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([1; 32]);
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng_2);
        assert_eq!(s1, s1_2);
        assert_eq!(s2, s2_2);
    }
}

#[test]
fn we_can_create_different_rand_F_vecs_from_different_seeds() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([2; 32]);
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng_2);
        assert_ne!(s1, s1_2);
        assert_ne!(s2, s2_2);
    }
}
