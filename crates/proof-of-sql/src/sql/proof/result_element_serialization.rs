use super::QueryError;
use crate::base::encode::VarInt;
use alloc::{string::String, vec::Vec};
use core::str;

pub trait ProvableResultElement<'a> {
    fn required_bytes(&self) -> usize;
    fn encode(&self, out: &mut [u8]) -> usize;

    fn decode(data: &'a [u8]) -> Result<(Self, usize), QueryError>
    where
        Self: Sized;
}

/// Implement encode and decode for integer types
impl<T: VarInt> ProvableResultElement<'_> for T {
    fn required_bytes(&self) -> usize {
        self.required_space()
    }

    fn encode(&self, out: &mut [u8]) -> usize {
        self.encode_var(out)
    }

    fn decode(data: &[u8]) -> Result<(Self, usize), QueryError> {
        VarInt::decode_var(data).ok_or(QueryError::Overflow)
    }
}

/// Implement encode for u8 buffer arrays
impl<'a> ProvableResultElement<'a> for &'a [u8] {
    fn required_bytes(&self) -> usize {
        self.len() + self.len().required_space()
    }

    fn encode(&self, out: &mut [u8]) -> usize {
        let len_buf: usize = self.len();
        let sizeof_usize = len_buf.encode_var(out);
        let bytes_written = len_buf + sizeof_usize;
        out[sizeof_usize..bytes_written].clone_from_slice(self);

        bytes_written
    }
    fn decode(data: &'a [u8]) -> Result<(Self, usize), QueryError> {
        let (len_buf, sizeof_usize) =
            <usize>::decode_var(data).ok_or(QueryError::MiscellaneousDecodingError)?;

        let bytes_read = len_buf + sizeof_usize;

        if data.len() < bytes_read {
            return Err(QueryError::MiscellaneousDecodingError);
        }

        Ok((&data[sizeof_usize..bytes_read], bytes_read))
    }
}

/// Implement encode for strings
impl<'a> ProvableResultElement<'a> for &'a str {
    fn required_bytes(&self) -> usize {
        self.as_bytes().required_bytes()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        self.as_bytes().encode(out)
    }
    fn decode(data: &'a [u8]) -> Result<(Self, usize), QueryError> {
        let (data, bytes_read) = <&[u8]>::decode(data)?;

        // arrow::array::StringArray only supports strings
        // whose maximum length (in bytes) is represented by a i32.
        // If we try to pass some string not respecting this restriction,
        // StringArray will panic. So we add this restriction here to
        // prevent this scenario.
        if data.len() > i32::MAX as usize {
            return Err(QueryError::MiscellaneousDecodingError);
        }

        Ok((
            str::from_utf8(data).map_err(|_e| QueryError::InvalidString)?,
            bytes_read,
        ))
    }
}

/// Implement encode for strings
impl ProvableResultElement<'_> for String {
    fn required_bytes(&self) -> usize {
        self.as_str().required_bytes()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        self.as_str().encode(out)
    }
    fn decode(data: &[u8]) -> Result<(Self, usize), QueryError> {
        decode_and_convert::<&str, String>(data)
    }
}

pub fn decode_and_convert<'a, F, T>(data: &'a [u8]) -> Result<(T, usize), QueryError>
where
    F: ProvableResultElement<'a>,
    T: From<F>,
{
    let (val, num_read) = F::decode(data)?;
    Ok((val.into(), num_read))
}

/// Implement the decode operation for multiple rows
pub fn decode_multiple_elements<'a, T: ProvableResultElement<'a>>(
    data: &'a [u8],
    n: usize,
) -> Result<(Vec<T>, usize), QueryError> {
    let mut res = Vec::with_capacity(n);
    let mut cnt = 0;
    for _ in 0..n {
        let (val, num_read) = <T>::decode(&data[cnt..])?;

        res.push(val);
        cnt += num_read;
    }

    Ok((res, cnt))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::base::scalar::Curve25519Scalar;
    use rand::{
        distributions::{Distribution, Uniform},
        rngs::StdRng,
    };
    use rand_core::SeedableRng;

    #[test]
    fn we_can_encode_and_decode_empty_buffers() {
        let mut out = vec![0_u8; 0_usize.required_space()];
        let empty_buf: &[u8] = &[][..];
        assert_eq!(empty_buf.required_bytes(), 0_usize.required_space());
        empty_buf.encode(&mut out[..]);
        let (decoded_buf, read_bytes) = <&[u8]>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_buf, empty_buf);
    }

    #[test]
    fn we_can_encode_and_decode_empty_strings() {
        let mut out = vec![0_u8; 0_usize.required_space()];
        let empty_string = "";
        assert_eq!(
            empty_string.as_bytes().required_bytes(),
            0_usize.required_space()
        );
        empty_string.as_bytes().encode(&mut out[..]);
        let (decoded_buf, read_bytes) = <&str>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_buf, empty_string);
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_integer() {
        let value = 123_i64;
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) = <i64>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value);
    }

    #[test]
    fn we_can_encode_and_decode_a_128_bit_integer() {
        let value = 123_i128;
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) = <i128>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value);
    }
    #[test]
    fn we_cannnot_decode_a_128_bit_integer_that_is_out_of_range() {
        let value = Curve25519Scalar::from(i128::MAX) + Curve25519Scalar::from(1);
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        assert!(matches!(
            <i128>::decode(&out[..]),
            Err(QueryError::Overflow)
        ));

        let value = Curve25519Scalar::from(i128::MIN) - Curve25519Scalar::from(1);
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        assert!(matches!(
            <i128>::decode(&out[..]),
            Err(QueryError::Overflow)
        ));
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_string() {
        let value = "test string";
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) = <&str>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value);
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_array() {
        let value = &[1_u8, 3_u8, 5_u8][..];
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) = <&[u8]>::decode(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value);
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_integer_to_a_scalar() {
        let value = 123_i64;
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) = Curve25519Scalar::decode_var(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value.into());
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_string_to_a_scalar() {
        let value = "test string";
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) =
            decode_and_convert::<&str, Curve25519Scalar>(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value.into());
    }

    #[test]
    fn we_can_encode_and_decode_a_simple_array_to_a_scalar() {
        let value = &[1_u8, 3_u8, 5_u8][..];
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        let (decoded_value, read_bytes) =
            decode_and_convert::<&[u8], Curve25519Scalar>(&out[..]).unwrap();
        assert_eq!(read_bytes, out.len());
        assert_eq!(decoded_value, value.into());
    }

    #[test]
    fn arbitrary_encoded_integers_are_correctly_decoded() {
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, usize::MAX);

        for _ in 0..100 {
            let value = match dist.sample(&mut rng).try_into() {
                Ok(val) => val,
                Err(_) => i64::MAX,
            };

            let mut out = vec![0_u8; value.required_bytes()];
            value.encode(&mut out[..]);

            let (decoded_value, read_bytes) = <i64>::decode(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value);

            let (decoded_value, read_bytes) = Curve25519Scalar::decode_var(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value.into());
        }
    }

    #[test]
    fn arbitrary_encoded_128_bit_integers_are_correctly_decoded() {
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(i128::MIN, i128::MAX);

        for _ in 0..100 {
            let value = dist.sample(&mut rng);

            let mut out = vec![0_u8; value.required_bytes()];
            value.encode(&mut out[..]);

            let (decoded_value, read_bytes) = <i128>::decode(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value);

            let (decoded_value, read_bytes) = Curve25519Scalar::decode_var(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value.into());
        }
    }

    #[test]
    fn arbitrary_encoded_strings_are_correctly_decoded() {
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, usize::MAX);

        for _ in 0..100 {
            let str = dist.sample(&mut rng).to_string()
                + "testing string encoding"
                    .repeat(dist.sample(&mut rng) % 100)
                    .as_str();
            let str_slice = str.as_str();

            let mut out = vec![0_u8; str_slice.required_bytes()];
            str_slice.encode(&mut out[..]);

            let (decoded_value, read_bytes) = <&str>::decode(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, str_slice);

            let (decoded_value, read_bytes) =
                decode_and_convert::<&str, Curve25519Scalar>(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, str_slice.into());
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[test]
    fn arbitrary_encoded_buffers_are_correctly_decoded() {
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, usize::MAX);

        for _ in 0..100 {
            let value = (0..(dist.sample(&mut rng) % 100))
                .map(|_v| (dist.sample(&mut rng) % 255) as u8)
                .collect::<Vec<u8>>();
            let value_slice = &value[..];

            let mut out = vec![0_u8; value_slice.required_bytes()];
            value_slice.encode(&mut out[..]);

            let (decoded_value, read_bytes) = <&[u8]>::decode(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value_slice);

            let (decoded_value, read_bytes) =
                decode_and_convert::<&[u8], Curve25519Scalar>(&out[..]).unwrap();
            assert_eq!(read_bytes, out.len());
            assert_eq!(decoded_value, value_slice.into());
        }
    }

    fn encode_multiple_rows<'a, T: ProvableResultElement<'a>>(data: &[T]) -> Vec<u8> {
        let total_len = data
            .iter()
            .map(super::ProvableResultElement::required_bytes)
            .sum::<usize>();

        let mut offset = 0;
        let mut out = vec![0_u8; total_len];
        for v in data {
            offset += v.encode(&mut out[offset..]);
        }

        out
    }

    #[test]
    fn multiple_integer_rows_are_correctly_encoded_and_decoded() {
        let data = [121_i64, -345_i64, 666_i64, 0_i64, i64::MAX, i64::MIN];
        let out = encode_multiple_rows(&data);
        let (decoded_data, decoded_bytes) =
            decode_multiple_elements::<i64>(&out[..], data.len()).unwrap();

        assert_eq!(decoded_data, data);
        assert_eq!(decoded_bytes, out.len());
    }

    #[test]
    fn multiple_128_bit_integer_rows_are_correctly_encoded_and_decoded() {
        let data = [121_i128, -345_i128, 666_i128, 0_i128, i128::MAX, i128::MIN];
        let out = encode_multiple_rows(&data);
        let (decoded_data, decoded_bytes) =
            decode_multiple_elements::<i128>(&out[..], data.len()).unwrap();

        assert_eq!(decoded_data, data);
        assert_eq!(decoded_bytes, out.len());
    }

    #[test]
    fn multiple_string_rows_are_correctly_encoded_and_decoded() {
        let data = ["abc1", "joe123", "testing435t"];
        let out = encode_multiple_rows(&data);
        let (decoded_data, decoded_bytes) =
            decode_multiple_elements::<&str>(&out[..], data.len()).unwrap();
        assert_eq!(decoded_data, data);
        assert_eq!(decoded_bytes, out.len());
    }

    #[test]
    fn multiple_array_rows_are_correctly_encoded_and_decoded() {
        let data = [
            &[121_u8, 0_u8, 39_u8, 93_u8][..],
            &[121_u8, 3_u8, 27_u8, 0_u8][..],
            &[121_u8, 7_u8, 111_u8, 45_u8][..],
        ];
        let out = encode_multiple_rows(&data);
        let (decoded_data, decoded_bytes) =
            decode_multiple_elements::<&[u8]>(&out[..], data.len()).unwrap();
        assert_eq!(decoded_data, data);
        assert_eq!(decoded_bytes, out.len());
    }

    #[test]
    fn empty_buffers_will_fail_to_decode_to_integers() {
        let value = 123_i64;
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);

        assert!(<i64>::decode(&out[..]).is_ok());
        assert!(<i64>::decode(&[]).is_err());
    }

    #[test]
    fn buffers_with_all_bits_set_will_fail_to_decode_to_integers() {
        let value = 123_i64;
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);

        assert!(<i64>::decode(&out[..]).is_ok());

        out[..].clone_from_slice(&vec![0b1111_1111; value.required_bytes()]);

        assert!(<i64>::decode(&out[..]).is_err());
    }

    #[test]
    fn buffers_with_invalid_utf8_characters_will_fail_to_decode_to_strings() {
        let value = "test_string";
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);

        assert!(<&str>::decode(&out[..]).is_ok());

        let last_element = out.len();
        out[last_element - 3..last_element].clone_from_slice(&[0xed, 0xa0, 0x80]);
        assert!(<&str>::decode(&out[..]).is_err());
    }

    #[test]
    fn buffers_smaller_than_sizeof_usize_will_fail_to_decode() {
        let value: &[u8] = &[][..];
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        assert_eq!(out.len(), value.len().required_space());
        assert!(<&[u8]>::decode(&out[..0]).is_err());
    }

    #[test]
    fn buffers_with_the_first_sizeof_usize_bytes_with_value_bigger_than_the_buffer_size_will_fail_to_decode(
    ) {
        let value = &[43_u8, 27_u8, 1_u8][..];
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        assert_eq!(out.len(), value.len().required_space() + value.len());
        assert!(<&[u8]>::decode(&out[..]).is_ok());

        assert_eq!(
            (value.len() + 1).required_space(),
            value.len().required_space()
        );
        (value.len() + 1).encode_var(&mut out[..]);
        assert!(<&[u8]>::decode(&out[..]).is_err());
    }

    #[test]
    fn buffers_with_the_first_sizeof_usize_bytes_with_value_smaller_than_the_buffer_size_will_not_fail_to_decode(
    ) {
        let value = &[43_u8, 27_u8, 1_u8][..];
        let mut out = vec![0_u8; value.required_bytes()];
        value.encode(&mut out[..]);
        assert_eq!(out.len(), value.len().required_space() + value.len());
        assert!(<&[u8]>::decode(&out[..]).is_ok());

        assert_eq!(
            value.len().required_space(),
            (value.len() - 1).required_space()
        );
        (value.len() - 1).encode_var(&mut out[..]);

        let expected_element = (
            &value[0..value.len() - 1],
            (value.len() - 1).required_space() + value.len() - 1,
        );
        assert_eq!(<&[u8]>::decode(&out[..]).unwrap(), expected_element);
    }

    #[test]
    fn decode_multiple_elements_will_fail_under_non_utf8_buffer_strings() {
        let data = [&[b'A', b'B', b'C'][..], &[0xed, 0xa0, 0x80][..]];

        let out = encode_multiple_rows(&data);

        let read_column = decode_multiple_elements::<&str>(&out[..], 1).unwrap();
        assert_eq!(read_column.0, vec!["ABC"]);
        assert_eq!(read_column.1, "ABC".required_bytes());

        assert!(decode_multiple_elements::<&str>(&out[..], 2).is_err());
    }

    #[test]
    fn decode_multiple_elements_will_fail_when_buffer_has_less_elements_than_specified() {
        let data = [&[b'A', b'B', b'C'][..], &[0xed, 0xa0, 0x80][..]];

        let out = encode_multiple_rows(&data);

        let read_column = decode_multiple_elements::<&[u8]>(&out[..], data.len()).unwrap();
        assert_eq!(read_column.0, data.to_vec());
        assert_eq!(read_column.1, out.len());

        assert!(decode_multiple_elements::<&str>(&out[..], data.len() + 1).is_err());
    }

    #[test]
    fn decode_multiple_elements_will_fail_under_invalid_buffers() {
        let data = [&[b'A', b'B', b'C'][..], &[b'A', b'B', b'C'][..]];

        let mut out = encode_multiple_rows(&data);

        let read_column = decode_multiple_elements::<&[u8]>(&out[..], data.len()).unwrap();
        assert_eq!(read_column.0, data.to_vec());
        assert_eq!(read_column.1, out.len());

        // we remove last element
        assert!(decode_multiple_elements::<&str>(&out[..out.len() - 1], data.len()).is_err());

        // we change the amount of elements specified in the buffer to be `data[1].len() + 1`
        assert_eq!(
            (data[1].len() + 1).required_space(),
            data[1].len().required_space()
        );
        (data[1].len() + 1).encode_var(&mut out[data[0].required_bytes()..]);
        assert!(decode_multiple_elements::<&str>(&out[..], data.len()).is_err());
    }

    #[test]
    fn we_cannot_decode_strings_with_more_than_i32_bytes() {
        let s_len = i32::MAX as usize + 1_usize;
        let mut s = vec![b'A'; s_len + s_len.required_space()];

        assert_eq!((s_len - 1_usize).required_space(), s_len.required_space());
        (s_len - 1_usize).encode_var(&mut s[..]);
        assert!(
            <&str>::decode(&s[..(s_len - 1_usize + (s_len - 1_usize).required_space())]).is_ok()
        );

        s_len.encode_var(&mut s[..]);
        assert!(<&str>::decode(&s[..]).is_err());
    }
}
