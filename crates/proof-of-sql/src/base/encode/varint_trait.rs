/**
 * Adapted from integer-encoding-rs
 *
 * See third_party/license/integer-encoding.LICENSE
 */
// ---------------------------------------------------------------------------------------------------------------
// The following chunck of code is copied from the `integer-encoding`. This is for two reasons:
// 1) it makes the `VarInt` no longer a foreign trait
// 2) there is a bug in `integer-encoding` that made it so that large decodings didn't fail when they should have
// There were significant code changes to simplify the code
// ---------------------------------------------------------------------------------------------------------------
use super::{
    scalar_varint::{
        read_scalar_varint, read_u256_varint, scalar_varint_size, u256_varint_size,
        write_scalar_varint, write_u256_varint,
    },
    U256,
};
use crate::base::scalar::MontScalar;
#[cfg(test)]
use alloc::{vec, vec::Vec};
use ark_ff::MontConfig;

/// Most-significant byte, == 0x80
pub const MSB: u8 = 0b1000_0000;
/// All bits except for the most significant. Can be used as bitmask to drop the most-signficant
/// bit using `&` (binary-and).
const DROP_MSB: u8 = 0b0111_1111;

/// Varint (variable length integer) encoding, as described in
/// <https://developers.google.com/protocol-buffers/docs/encoding>.
///
/// Uses zigzag encoding (also described there) for signed integer representation.
pub trait VarInt: Sized + Copy {
    /// Returns the number of bytes this number needs in its encoded form. Note: This varies
    /// depending on the actual number you want to encode.
    fn required_space(self) -> usize;
    /// Decode a value from the slice. Returns the value and the number of bytes read from the
    /// slice (can be used to read several consecutive values from a big slice)
    /// return None if the decoded value overflows this type.
    fn decode_var(src: &[u8]) -> Option<(Self, usize)>;
    /// Encode a value into the slice. The slice must be at least `required_space()` bytes long.
    /// The number of bytes taken by the encoded integer is returned.
    fn encode_var(self, src: &mut [u8]) -> usize;

    /// Helper: Encode a value and return the encoded form as Vec. The Vec must be at least
    /// `required_space()` bytes long.
    #[cfg(test)]
    fn encode_var_vec(self) -> Vec<u8> {
        let mut v = vec![0; self.required_space()];
        self.encode_var(&mut v);
        v
    }
}

#[inline]
fn zigzag_encode(from: i64) -> u64 {
    ((from << 1) ^ (from >> 63)) as u64
}

// see: http://stackoverflow.com/a/2211086/56332
// casting required because operations like unary negation
// cannot be performed on unsigned integers
#[inline]
fn zigzag_decode(from: u64) -> i64 {
    ((from >> 1) ^ (-((from & 1) as i64)) as u64) as i64
}

/// TODO: add docs
macro_rules! impl_varint {
    ($t:ty, unsigned) => {
        impl VarInt for $t {
            fn required_space(self) -> usize {
                (self as u64).required_space()
            }

            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = u64::decode_var(src)?;
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
                if n > (Self::MAX as u64) {
                    None
                } else {
                    Some((n as Self, s))
                }
            }

            fn encode_var(self, dst: &mut [u8]) -> usize {
                (self as u64).encode_var(dst)
            }
        }
    };
    ($t:ty, signed) => {
        impl VarInt for $t {
            fn required_space(self) -> usize {
                (self as i64).required_space()
            }

            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = i64::decode_var(src)?;
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
                if n > (Self::MAX as i64) || n < (Self::MIN as i64) {
                    None
                } else {
                    Some((n as Self, s))
                }
            }

            fn encode_var(self, dst: &mut [u8]) -> usize {
                (self as i64).encode_var(dst)
            }
        }
    };
}

impl_varint!(usize, unsigned);
impl_varint!(u32, unsigned);
impl_varint!(u16, unsigned);
impl_varint!(u8, unsigned);

impl_varint!(isize, signed);
impl_varint!(i32, signed);
impl_varint!(i16, signed);
impl_varint!(i8, signed);

impl VarInt for bool {
    fn required_space(self) -> usize {
        (self as u64).required_space()
    }

    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let (n, s) = u64::decode_var(src)?;
        // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
        match n {
            0 => Some((false, s)),
            1 => Some((true, s)),
            _ => None,
        }
    }

    fn encode_var(self, dst: &mut [u8]) -> usize {
        (self as u64).encode_var(dst)
    }
}

// Below are the "base implementations" doing the actual encodings; all other integer types are
// first cast to these biggest types before being encoded.

impl VarInt for u64 {
    fn required_space(self) -> usize {
        let bits = 64 - self.leading_zeros() as usize;
        core::cmp::max(1, (bits + 6) / 7)
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let mut result: u64 = 0;
        let mut shift = 0;

        let mut success = false;
        for b in src {
            let msb_dropped = b & DROP_MSB;
            result |= (msb_dropped as u64) << shift;
            shift += 7;

            if shift > (9 * 7) {
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `u64`.
                success = *b < 2;
                break;
            } else if b & MSB == 0 {
                success = true;
                break;
            }
        }

        if success {
            Some((result, shift / 7))
        } else {
            None
        }
    }

    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        assert!(dst.len() >= self.required_space());
        let mut n = self;
        let mut i = 0;

        while n >= 0x80 {
            dst[i] = MSB | (n as u8);
            i += 1;
            n >>= 7;
        }

        dst[i] = n as u8;
        i + 1
    }
}

impl VarInt for i64 {
    fn required_space(self) -> usize {
        zigzag_encode(self).required_space()
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let (result, size) = u64::decode_var(src)?;
        Some((zigzag_decode(result), size))
    }

    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        zigzag_encode(self).encode_var(dst)
    }
}

impl VarInt for U256 {
    fn required_space(self) -> usize {
        u256_varint_size(self)
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        read_u256_varint(src)
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        write_u256_varint(dst, self)
    }
}

impl VarInt for u128 {
    fn required_space(self) -> usize {
        U256 { low: self, high: 0 }.required_space()
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        match U256::decode_var(src)? {
            (U256 { high: 0, low }, s) => Some((low, s)),
            _ => None,
        }
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        U256 { low: self, high: 0 }.encode_var(dst)
    }
}

// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
#[inline]
fn zigzag_encode_i128(from: i128) -> u128 {
    ((from << 1) ^ (from >> 127)) as u128
}
// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
// see: http://stackoverflow.com/a/2211086/56332
// casting required because operations like unary negation
// cannot be performed on unsigned integers
#[inline]
fn zigzag_decode_i128(from: u128) -> i128 {
    ((from >> 1) ^ (-((from & 1) as i128)) as u128) as i128
}
impl VarInt for i128 {
    fn required_space(self) -> usize {
        u128::required_space(zigzag_encode_i128(self))
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        u128::decode_var(src).map(|(v, s)| (zigzag_decode_i128(v), s))
    }

    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        zigzag_encode_i128(self).encode_var(dst)
    }
}

impl<T: MontConfig<4>> VarInt for MontScalar<T> {
    fn required_space(self) -> usize {
        scalar_varint_size(&self)
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        read_scalar_varint(src)
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        write_scalar_varint(dst, &self)
    }
}
