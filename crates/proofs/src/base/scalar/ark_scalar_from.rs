use super::Scalar;
use crate::base::scalar::ArkScalar;
use ark_curve25519::Fr as F;
use ark_ff::PrimeField;
use arrow::datatypes::i256;
use num_traits::Zero;

macro_rules! impl_from_for_ark_scalar_for_type_supported_by_from {
    ($tt:ty) => {
        impl From<$tt> for ArkScalar {
            fn from(x: $tt) -> Self {
                ArkScalar(x.into())
            }
        }
    };
}
impl From<&[u8]> for ArkScalar {
    fn from(x: &[u8]) -> Self {
        if x.is_empty() {
            return ArkScalar::zero();
        }

        let hash = blake3::hash(x);
        let mut bytes: [u8; 32] = hash.into();
        bytes[31] &= 0b00001111_u8;

        ArkScalar::from_le_bytes_mod_order(&bytes)
    }
}
macro_rules! impl_from_for_ark_scalar_for_string {
    ($tt:ty) => {
        impl From<$tt> for ArkScalar {
            fn from(x: $tt) -> Self {
                x.as_bytes().into()
            }
        }
    };
}

impl_from_for_ark_scalar_for_type_supported_by_from!(bool);
impl_from_for_ark_scalar_for_type_supported_by_from!(u8);
impl_from_for_ark_scalar_for_type_supported_by_from!(u16);
impl_from_for_ark_scalar_for_type_supported_by_from!(u32);
impl_from_for_ark_scalar_for_type_supported_by_from!(u64);
impl_from_for_ark_scalar_for_type_supported_by_from!(u128);
impl_from_for_ark_scalar_for_type_supported_by_from!(i8);
impl_from_for_ark_scalar_for_type_supported_by_from!(i16);
impl_from_for_ark_scalar_for_type_supported_by_from!(i32);
impl_from_for_ark_scalar_for_type_supported_by_from!(i64);
impl_from_for_ark_scalar_for_type_supported_by_from!(i128);
impl_from_for_ark_scalar_for_string!(&str);
impl_from_for_ark_scalar_for_string!(String);

impl<T> From<&T> for ArkScalar
where
    T: Into<ArkScalar> + Clone,
{
    fn from(x: &T) -> Self {
        x.clone().into()
    }
}

impl From<ArkScalar> for i256 {
    // ArkScalar is 252 bits and so will always fit inside of i256
    fn from(val: ArkScalar) -> i256 {
        let is_negative = val > Scalar::MAX_SIGNED;
        let abs_scalar = if is_negative { -val } else { val };
        let limbs = abs_scalar.0.into_bigint().0;

        let low = (limbs[0] as u128) | ((limbs[1] as u128) << 64);
        let high = (limbs[2] as i128) | ((limbs[3] as i128) << 64);

        let abs_i256 = i256::from_parts(low, high);
        if is_negative {
            i256::wrapping_neg(abs_i256)
        } else {
            abs_i256
        }
    }
}

#[derive(Debug)]
pub enum ArkScalarConversionError {
    ValueTooLarge,
    ValueTooSmall,
}

impl TryFrom<i256> for ArkScalar {
    type Error = ArkScalarConversionError;

    // Must fit inside 252 bits and so requires fallible
    fn try_from(value: i256) -> Result<Self, Self::Error> {
        let bytes = value.wrapping_abs().to_le_bytes();

        match value.is_negative() {
            true => {
                if value < -(i256::from(ArkScalar::MAX_SIGNED)) {
                    return Err(ArkScalarConversionError::ValueTooSmall);
                }
                let field_element = F::from_le_bytes_mod_order(&bytes);
                Ok(ArkScalar(F::zero() - field_element))
            }
            false => {
                if value > ArkScalar::MAX_SIGNED.into() {
                    Err(ArkScalarConversionError::ValueTooLarge)
                } else {
                    Ok(ArkScalar::from_le_bytes_mod_order(&bytes))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::base::scalar::{random_i256, ArkScalar, Scalar};
    use arrow::datatypes::i256;
    use num_traits::Zero;

    const MIN_SUPPORTED_I256_STR: &str =
        "-3618502788666131106986593281521497120428558179689953803000975469142727125494";
    const MAX_SUPPORTED_I256_STR: &str =
        "3618502788666131106986593281521497120428558179689953803000975469142727125494";

    #[test]
    fn test_arkscalar_to_i256_conversion() {
        let positive_scalar = ArkScalar::from(12345);
        let expected_i256 = i256::from(12345);
        assert_eq!(i256::from(positive_scalar), expected_i256);

        let negative_scalar = ArkScalar::from(-12345);
        let expected_i256 = i256::from(-12345);
        assert_eq!(i256::from(negative_scalar), expected_i256);

        let max_scalar = ArkScalar::MAX_SIGNED;
        let expected_max = i256::from(ArkScalar::MAX_SIGNED);
        assert_eq!(i256::from(max_scalar), expected_max);

        let min_scalar = ArkScalar::from(0);
        let expected_min = i256::from(ArkScalar::from(0));
        assert_eq!(i256::from(min_scalar), expected_min);
    }

    #[test]
    fn test_arkscalar_i256_overflow_and_underflow() {
        // 2^256 overflows
        assert!(ArkScalar::try_from(i256::MAX).is_err());

        // MAX_SIGNED + 1 overflows
        assert!(ArkScalar::try_from(
            i256::from_string(MAX_SUPPORTED_I256_STR).unwrap() + i256::from(1)
        )
        .is_err());

        // -2^255 underflows
        assert!(i256::MIN < -(i256::from(ArkScalar::MAX_SIGNED)));
        assert!(ArkScalar::try_from(i256::MIN).is_err());

        // -MAX-SIGNED - 1 underflows
        assert!(ArkScalar::try_from(
            i256::from_string(MIN_SUPPORTED_I256_STR).unwrap() - i256::from(1)
        )
        .is_err());
    }

    #[test]
    fn test_i256_arkscalar_negative() {
        // Test conversion from i256(-1) to ArkScalar
        let neg_one_i256_arkscalar = ArkScalar::try_from(i256::from(-1));
        assert!(neg_one_i256_arkscalar.is_ok());
        let neg_one_arkscalar = ArkScalar::from(-1);
        assert_eq!(neg_one_i256_arkscalar.unwrap(), neg_one_arkscalar);
    }

    #[test]
    fn test_i256_arkscalar_zero() {
        // Test conversion from i256(0) to ArkScalar
        let zero_i256_arkscalar = ArkScalar::try_from(i256::from(0));
        assert!(zero_i256_arkscalar.is_ok());
        let zero_arkscalar = ArkScalar::zero();
        assert_eq!(zero_i256_arkscalar.unwrap(), zero_arkscalar);
    }

    #[test]
    fn test_i256_arkscalar_positive() {
        // Test conversion from i256(42) to ArkScalar
        let forty_two_i256_arkscalar = ArkScalar::try_from(i256::from(42));
        let forty_two_arkscalar = ArkScalar::from(42);
        assert_eq!(forty_two_i256_arkscalar.unwrap(), forty_two_arkscalar);
    }

    #[test]
    fn test_i256_arkscalar_max_signed() {
        let max_signed = i256::from_string(MAX_SUPPORTED_I256_STR);
        assert!(max_signed.is_some());
        // max signed value
        let max_signed_scalar = ArkScalar::MAX_SIGNED;
        // Test conversion from i256 to ArkScalar
        let i256_arkscalar = ArkScalar::try_from(max_signed.unwrap());
        assert!(i256_arkscalar.is_ok());
        assert_eq!(i256_arkscalar.unwrap(), max_signed_scalar);
    }

    #[test]
    fn test_i256_arkscalar_min_signed() {
        let min_signed = i256::from_string(MIN_SUPPORTED_I256_STR);
        assert!(min_signed.is_some());
        let i256_arkscalar = ArkScalar::try_from(min_signed.unwrap());
        // -MAX_SIGNED is ok
        assert!(i256_arkscalar.is_ok());
        assert_eq!(
            i256_arkscalar.unwrap(),
            ArkScalar::MAX_SIGNED + ArkScalar::from(1)
        );
    }

    #[test]
    fn test_i256_arkscalar_random() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let i256_value = random_i256(&mut rng);
            let ark_scalar = ArkScalar::try_from(i256_value).expect("Conversion failed");
            let back_to_i256 = i256::from(ark_scalar);
            assert_eq!(i256_value, back_to_i256, "Round-trip conversion failed");
        }
    }
}

#[cfg(test)]
impl_from_for_ark_scalar_for_type_supported_by_from!(ark_ff::BigInt<4>);
