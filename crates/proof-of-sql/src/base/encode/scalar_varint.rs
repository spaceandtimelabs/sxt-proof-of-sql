use crate::base::{
    encode::{ZigZag, U256},
    scalar::MontScalar,
};
use ark_ff::MontConfig;
use core::cmp::{max, Ordering};

/// This function writes the input scalar x as a varint encoding to buf slice
///
/// See `<https://developers.google.com/protocol-buffers/docs/encoding#varints>` as reference.
///
/// return:
/// - the total number of bytes N written to buf
///
/// crash:
/// - in case N is bigger than `buf.len()`
pub fn write_scalar_varint<T: MontConfig<4>>(buf: &mut [u8], x: &MontScalar<T>) -> usize {
    write_u256_varint(buf, x.zigzag())
}
pub fn write_u256_varint(buf: &mut [u8], mut zig_x: U256) -> usize {
    let mut pos = 0;

    // we keep writing until we get a value that has the MSB not set.
    // a MSB not set implies that we have reached the end of the number.
    while zig_x.high != 0 || zig_x.low >= 0b1000_0000 {
        // we read the next 7 bits from `zig_x` casting to u8 and setting
        // the 8-th bit to 1 to indicate that we still need to write more bytes to buf
        buf[pos] = (zig_x.low as u8) | 0b1000_0000;
        pos += 1;

        // we shift the whole `zig_x` number 7 bits to right
        zig_x.low = (zig_x.low >> 7) | ((zig_x.high & 0b0111_1111) << 121);
        zig_x.high >>= 7;
    }

    // we write the last byte to buf with the MSB not set.
    // that indicates that the number has no continuation.
    buf[pos] = (zig_x.low & 0b0111_1111) as u8;

    pos + 1
}

/// This function consumes the N first byte elements from buf slice
/// that have their MSB set plus 1 more byte that does not have the MSB set.
/// These consumed bytes must represent a varint encoded number. Effectively,
/// each byte can have up to 7-bit set associated with the encoded number,
/// besides MSB 1-bit to represent in which byte the encoding ends.
///
/// return `Some((value, read_bytes))`:
/// - `value` = the dalek scalar generated out of the consumed bytes
/// - `read_bytes` = the total number of bytes N consumed
///
/// return None:
/// - in case of more than 37 bytes are read
/// - in case of more bytes read than the buffer length
///
/// Note: because this function can read up to 37 bytes,
///  buf can represent a number with up to 37 * 7 bits = 259 bits.
///  Since read-scalar stores the buf into a U256 type, which can only
///  hold up to 256 bit numbers, the non-continuation bits
///  257 up to 259 from buf are ignored.
pub fn read_scalar_varint<T: MontConfig<4>>(buf: &[u8]) -> Option<(MontScalar<T>, usize)> {
    read_u256_varint(buf).map(|(val, s)| (val.zigzag(), s))
}
pub fn read_u256_varint(buf: &[u8]) -> Option<(U256, usize)> {
    // The decoded value representing a u256 integer
    let mut val = U256::from_words(0, 0);

    // The number of bits to shift by (<<0, <<7, <<14, etc)
    let mut shift_amount: u32 = 0;

    // we keep reading until we find a byte with the MSB equal to zero,
    // which implies that we have read the whole varint number
    for next_byte in buf {
        // we write the `next 7 bits` at the [shift_amount..shift_amount + 7)
        // bit positions of val u256 number
        match shift_amount.cmp(&126_u32) {
            Ordering::Less => val.low |= ((*next_byte & 0b0111_1111) as u128) << shift_amount,
            Ordering::Equal => {
                val.low |= ((*next_byte & 0b0000_0011) as u128) << shift_amount;
                val.high |= ((*next_byte & 0b0111_1100) as u128) >> 2;
            }
            Ordering::Greater => {
                val.high |= ((*next_byte & 0b0111_1111) as u128) << (shift_amount - 128);
            }
        }

        shift_amount += 7;

        if (*next_byte >> 7) == 0 {
            // check if we have reached the end of the encoding (MSB not set)
            return Some((val, (shift_amount / 7) as usize));
        }

        if shift_amount > 256 {
            // the dalek scalar can only support 256 bits
            return None;
        }
    }

    // we read all the bytes in buf, but couldn't reach the end of the varint encoding
    None
}

/// This function writes all the input scalars `scals` to the input buffer `buf`.
/// For that, the Varint together with the [`ZigZag`] encoding is used.
///
/// return:
/// - the total number of bytes written to buf
///
/// error:
/// - in case buf has not enough space to hold all the scalars encoding.
#[cfg(test)]
pub fn write_scalar_varints<T: MontConfig<4>>(buf: &mut [u8], scals: &[MontScalar<T>]) -> usize {
    let mut total_bytes_written = 0;

    for scal in scals {
        let bytes_written = write_scalar_varint(&mut buf[total_bytes_written..], scal);

        total_bytes_written += bytes_written;
    }

    total_bytes_written
}

/// This function read all the specified scalars from `input_buf` to `scals_buf`.
/// For that, it converts the input buffer from a Varint and [`ZigZag`] encoding to a Dalek Scalar
///
/// See `<https://developers.google.com/protocol-buffers/docs/encoding#varints>` as reference.
///
/// error:
/// - in case it's not possible to read all specified scalars from `input_buf`
#[cfg(test)]
pub fn read_scalar_varints<T: MontConfig<4>>(
    scals_buf: &mut [MontScalar<T>],
    input_buf: &[u8],
) -> Option<()> {
    let mut buf = input_buf;

    for scal_buf in scals_buf.iter_mut() {
        let (scal, bytes_read) = read_scalar_varint(buf)?;

        *scal_buf = scal;
        buf = &buf[bytes_read..];
    }

    Some(())
}

/// This function returns the varint encoding size for the given scalar
///
/// This function should be used to get an upper bound on the buffer size
/// used by the `write_scalar_varint` function.
pub fn scalar_varint_size<T: MontConfig<4>>(x: &MontScalar<T>) -> usize {
    u256_varint_size(x.zigzag())
}
pub fn u256_varint_size(zig_x: U256) -> usize {
    let zigzag_size = if zig_x.high == 0 {
        128 - zig_x.low.leading_zeros()
    } else {
        256 - zig_x.high.leading_zeros()
    };

    // we must at least return 1. because even for
    // the 0 scalar case, we need one byte for the encoding
    max(1, (zigzag_size as usize + 6) / 7)
}

/// This function returns the varint encoding size for the given scalar slice
///
/// This function should be used to get an upper bound on the buffer size
/// used by the `write_scalar_varints` function.
#[cfg(test)]
pub fn scalar_varints_size<T: MontConfig<4>>(scals: &[MontScalar<T>]) -> usize {
    let mut all_size: usize = 0;

    for x in scals {
        all_size += scalar_varint_size(x);
    }

    all_size
}
