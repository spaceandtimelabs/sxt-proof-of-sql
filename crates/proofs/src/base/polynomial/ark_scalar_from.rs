use crate::base::polynomial::ArkScalar;
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

#[cfg(test)]
impl_from_for_ark_scalar_for_type_supported_by_from!(ark_ff::BigInt<4>);
