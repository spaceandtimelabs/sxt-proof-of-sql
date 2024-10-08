use crate::base::scalar::MontScalar;
use alloc::string::String;
use ark_ff::MontConfig;
use num_traits::Zero;

/// TODO: add docs
macro_rules! impl_from_for_mont_scalar_for_type_supported_by_from {
    ($tt:ty) => {
        impl<T: MontConfig<4>> From<$tt> for MontScalar<T> {
            fn from(x: $tt) -> Self {
                Self(x.into())
            }
        }
    };
}
impl<T: MontConfig<4>> From<&[u8]> for MontScalar<T> {
    /// Converts a byte slice to a `MontScalar`.
    ///
    /// - If the byte slice is empty, the result is the zero scalar.
    /// - If the byte slice has length 31 or less, the bytes are directly converted to a scalar.
    /// - If the byte slice has length 32, the bytes are hashed using `blake3` and the result is
    ///   converted to a scalar.
    fn from(x: &[u8]) -> Self {
        match x.len() {
            0 => Self::zero(),
            1..=31 => {
                // Convert directly if 31 bytes or less
                let mut bytes = [0u8; 32];
                bytes[..x.len()].copy_from_slice(x);
                Self::from_le_bytes_mod_order(&bytes)
            }
            32 => {
                // Hash and convert if exactly 32 bytes
                let hash = blake3::hash(x);
                let mut bytes: [u8; 32] = hash.into();
                bytes[31] &= 0b0000_1111_u8;
                Self::from_le_bytes_mod_order(&bytes)
            }
            _ => panic!("Unsupported byte length for conversion to MontScalar"),
        }
    }
}
/// TODO: add docs
macro_rules! impl_from_for_mont_scalar_for_string {
    ($tt:ty) => {
        impl<T: MontConfig<4>> From<$tt> for MontScalar<T> {
            fn from(x: $tt) -> Self {
                let bytes = x.as_bytes();
                if bytes.is_empty() {
                    return Self::zero();
                }

                let hash = blake3::hash(bytes);
                let mut bytes: [u8; 32] = hash.into();
                bytes[31] &= 0b0000_1111_u8;

                Self::from_le_bytes_mod_order(&bytes)
            }
        }
    };
}

impl_from_for_mont_scalar_for_type_supported_by_from!(bool);
impl_from_for_mont_scalar_for_type_supported_by_from!(u8);
impl_from_for_mont_scalar_for_type_supported_by_from!(u16);
impl_from_for_mont_scalar_for_type_supported_by_from!(u32);
impl_from_for_mont_scalar_for_type_supported_by_from!(u64);
impl_from_for_mont_scalar_for_type_supported_by_from!(u128);
impl_from_for_mont_scalar_for_type_supported_by_from!(i8);
impl_from_for_mont_scalar_for_type_supported_by_from!(i16);
impl_from_for_mont_scalar_for_type_supported_by_from!(i32);
impl_from_for_mont_scalar_for_type_supported_by_from!(i64);
impl_from_for_mont_scalar_for_type_supported_by_from!(i128);
impl_from_for_mont_scalar_for_string!(&str);
impl_from_for_mont_scalar_for_string!(String);

impl<F: MontConfig<4>, T> From<&T> for MontScalar<F>
where
    T: Into<MontScalar<F>> + Clone,
{
    fn from(x: &T) -> Self {
        x.clone().into()
    }
}
