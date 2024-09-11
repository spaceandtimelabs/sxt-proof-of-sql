use ark_ff::{Fp, MontBackend, MontConfig};

/// This should only be used for the purpose of unit testing.
/// For now this is simply decorating ark_curve25519::FrConfig so as to make a struct that
/// functions the same but is viewed as a different struct to the compiler.
pub struct TestMontConfig(pub ark_curve25519::FrConfig);

impl TestMontConfig {
    const fn convert_curve_25519_backend_to_test_backend(
        backend: Fp<MontBackend<ark_curve25519::FrConfig, 4>, 4>,
    ) -> Fp<MontBackend<Self, 4>, 4> {
        Fp::new(backend.0)
    }
}

impl MontConfig<4> for TestMontConfig {
    const MODULUS: ark_ff::BigInt<4> = <ark_curve25519::FrConfig as MontConfig<4>>::MODULUS;

    const GENERATOR: Fp<MontBackend<Self, 4>, 4> =
        Self::convert_curve_25519_backend_to_test_backend(
            <ark_curve25519::FrConfig as MontConfig<4>>::GENERATOR,
        );

    const TWO_ADIC_ROOT_OF_UNITY: ark_ff::Fp<ark_ff::MontBackend<Self, 4>, 4> =
        Self::convert_curve_25519_backend_to_test_backend(
            <ark_curve25519::FrConfig as MontConfig<4>>::TWO_ADIC_ROOT_OF_UNITY,
        );
}
