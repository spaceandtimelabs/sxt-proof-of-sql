use crate::base::scalar::MontScalar;

/// The scalar used in the `HyperKZG` PCS. This is the BN254 scalar.
pub type BNScalar = MontScalar<ark_bn254::FrConfig>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar_constants;
    #[cfg(feature = "hyperkzg_proof")]
    use crate::base::scalar::Scalar;
    #[cfg(feature = "hyperkzg_proof")]
    use ark_std::UniformRand;
    #[cfg(feature = "hyperkzg_proof")]
    use nova_snark::provider::bn256_grumpkin::bn256::Scalar as NovaScalar;

    #[test]
    fn we_have_correct_constants_for_bn_scalar() {
        test_scalar_constants::<BNScalar>();
    }

    #[test]
    #[cfg(feature = "hyperkzg_proof")]
    fn we_can_convert_from_posql_scalar_to_nova_scalar() {
        // Test zero
        assert_eq!(NovaScalar::from(0_u64), NovaScalar::from(BNScalar::ZERO));

        // Test one
        assert_eq!(NovaScalar::from(1_u64), NovaScalar::from(BNScalar::ONE));

        // Test negative one
        assert_eq!(-NovaScalar::from(1_u64), NovaScalar::from(-BNScalar::ONE));

        // Test two
        assert_eq!(NovaScalar::from(2_u64), NovaScalar::from(BNScalar::TWO));

        // Test ten
        assert_eq!(NovaScalar::from(10_u64), NovaScalar::from(BNScalar::TEN));

        // Test a large value
        let large_value = BNScalar::from(123_456_789_u64);
        assert_eq!(
            NovaScalar::from(123_456_789_u64),
            NovaScalar::from(large_value)
        );

        let mut rng = ark_std::test_rng();

        for _ in 0..10 {
            let a = BNScalar::rand(&mut rng);
            let b = BNScalar::rand(&mut rng);
            assert_eq!(
                NovaScalar::from(a + b),
                NovaScalar::from(a) + NovaScalar::from(b)
            );
            assert_eq!(
                NovaScalar::from(a * b),
                NovaScalar::from(a) * NovaScalar::from(b)
            );
        }
    }
}
