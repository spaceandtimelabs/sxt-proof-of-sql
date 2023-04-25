use crate::base::polynomial::ArkScalar;
use ark_ff::PrimeField;
use curve25519_dalek::scalar::Scalar;
use num_traits::Zero;

/// Provides conversion to [ArkScalar].
///
/// This conversion is especially important for proofs.
/// Any data type we want to support will need to be able to convert to ArkScalar.
/// So, this trait may be used as a bound for supported data types.
///
/// We could just use rust's [From] and [Into] traits.
/// However, some types we want to support are foreign, and since ArkScalar itself is foreign, we
/// won't be able to provide these conversions.
///
/// One solution would be to create a new-type around every foreign type we want to support.
/// The other is to provide a new conversion trait entirely.
///
/// The latter was chosen for two reasons:
/// 1. We can still create new-types if we want to, but we don't have to in simple cases.
/// 2. There may be already-existing conversions for ArkScalar on types we *don't* want to support.
/// A new trait allows us to be explicit about the types we want to support.
pub trait ToArkScalar {
    fn to_ark_scalar(&self) -> ArkScalar;
}

macro_rules! impl_to_ark_scalar_for_type_supported_by_from {
    ($tt:ty) => {
        impl ToArkScalar for $tt {
            fn to_ark_scalar(&self) -> ArkScalar {
                ArkScalar::from(*self)
            }
        }
    };
}

impl ToArkScalar for Scalar {
    fn to_ark_scalar(&self) -> ArkScalar {
        crate::base::polynomial::to_ark_scalar(self)
    }
}

impl_to_ark_scalar_for_type_supported_by_from!(ArkScalar);
impl_to_ark_scalar_for_type_supported_by_from!(bool);
impl_to_ark_scalar_for_type_supported_by_from!(u8);
impl_to_ark_scalar_for_type_supported_by_from!(u16);
impl_to_ark_scalar_for_type_supported_by_from!(u32);
impl_to_ark_scalar_for_type_supported_by_from!(u64);
impl_to_ark_scalar_for_type_supported_by_from!(u128);
impl_to_ark_scalar_for_type_supported_by_from!(i8);
impl_to_ark_scalar_for_type_supported_by_from!(i16);
impl_to_ark_scalar_for_type_supported_by_from!(i32);
impl_to_ark_scalar_for_type_supported_by_from!(i64);
impl_to_ark_scalar_for_type_supported_by_from!(i128);

macro_rules! byte_array_to_ark_scalar {
    ($it:ty) => {
        impl ToArkScalar for $it {
            fn to_ark_scalar(&self) -> ArkScalar {
                if self.is_empty() {
                    return ArkScalar::zero();
                }

                let hash = blake3::hash(self);
                let mut bytes: [u8; 32] = hash.into();
                bytes[31] &= 0b00001111_u8;

                ArkScalar::from_le_bytes_mod_order(&bytes)
            }
        }
    };
}

byte_array_to_ark_scalar!([u8]);
byte_array_to_ark_scalar!(&[u8]);

macro_rules! string_to_ark_scalar {
    ($tt:ty) => {
        impl ToArkScalar for $tt {
            fn to_ark_scalar(&self) -> ArkScalar {
                self.as_bytes().to_ark_scalar()
            }
        }
    };
}

string_to_ark_scalar!(&str);
string_to_ark_scalar!(String);
string_to_ark_scalar!(&String);
