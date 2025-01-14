use super::MontScalar;
use ark_ff::{Fp, MontBackend, MontConfig};

/// An implementation of `Scalar` intended for use in testing when a concrete implementation is required.
///
/// Ultimately, a wrapper type around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
pub type TestScalar = MontScalar<TestMontConfig>;

/// An implementation of `MontConfig` intended for use in testing when a concrete implementation is required.
///
/// Ultimately, a wrapper type around the field element `ark_curve25519::FrConfig` and should be used in place of `ark_curve25519::FrConfig`.
pub struct TestMontConfig(pub ark_curve25519::FrConfig);

impl MontConfig<4> for TestMontConfig {
    const MODULUS: ark_ff::BigInt<4> = <ark_curve25519::FrConfig as MontConfig<4>>::MODULUS;

    const GENERATOR: Fp<MontBackend<Self, 4>, 4> =
        Fp::new(<ark_curve25519::FrConfig as MontConfig<4>>::GENERATOR.0);

    const TWO_ADIC_ROOT_OF_UNITY: ark_ff::Fp<ark_ff::MontBackend<Self, 4>, 4> =
        Fp::new(<ark_curve25519::FrConfig as MontConfig<4>>::TWO_ADIC_ROOT_OF_UNITY.0);
}
