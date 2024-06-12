use super::scalar_varint::{
    read_scalar_varint, read_scalar_varints, scalar_varint_size, scalar_varints_size,
    write_scalar_varint, write_scalar_varints,
};
use crate::base::{encode::U256, scalar::Curve25519Scalar};

#[test]
fn small_scalars_are_encoded_as_positive_varints_and_consume_few_bytes() {
    assert!(scalar_varint_size(&Curve25519Scalar::from(0_u64)) == 1);
    assert!(scalar_varint_size(&Curve25519Scalar::from(1_u64)) == 1);
    assert!(scalar_varint_size(&Curve25519Scalar::from(2_u64)) == 1);
    assert!(scalar_varint_size(&Curve25519Scalar::from(1000_u64)) == 2);
}

#[test]
fn big_scalars_with_small_additive_inverses_are_encoded_as_negative_varints_and_consume_few_bytes()
{
    // x = p - 1 (p is the ristretto group order)
    // y = -x = 1
    let val = -Curve25519Scalar::from(1_u64);
    assert!(scalar_varint_size(&val) == 1);

    // x = p - 1000 (p is the ristretto group order)
    // y = -x = 1000
    let val = -Curve25519Scalar::from(1000_u64);
    assert!(scalar_varint_size(&val) == 2);
}

#[test]
fn big_scalars_that_are_smaller_than_their_additive_inverses_are_encoded_as_positive_varints_and_consume_many_bytes(
) {
    // x = (p - 1) / 10 (p is the ristretto group order)
    // y = -x = (p + 1) / 10
    let val: Curve25519Scalar = (&U256::from_words(
        0x9bafe5c976b25c7bd59b704f6fb22eca,
        0x1999999999999999999999999999999,
    ))
        .into();
    assert!(scalar_varint_size(&val) == 36);
}

#[test]
fn big_additive_inverses_that_are_smaller_than_the_input_scalars_are_encoded_as_negative_varints_and_consume_many_bytes(
) {
    // x = (p + 1) / 10 (p is the ristretto group order)
    // y = -x = (p - 1) / 10
    let val: Curve25519Scalar = (&U256::from_words(
        0x9bafe5c976b25c7bd59b704f6fb22ecb,
        0x1999999999999999999999999999999,
    ))
        .into();
    assert!(scalar_varint_size(&val) == 36);
}

#[test]
fn the_maximum_positive_and_negative_encoded_scalars_consume_the_maximum_amount_of_bytes() {
    // maximum negative encoded scalar
    // x = (p + 1) / 2 (p is the ristretto group order)
    // y = -x = (p - 1) / 2
    let val: Curve25519Scalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f7,
        0x8000000000000000000000000000000,
    ))
        .into();
    assert!(scalar_varint_size(&val) == 37);

    // maximum positive encoded scalar
    // x = (p - 1) / 2 (p is the ristretto group order)
    // y = -x = (p + 1) / 2
    let val: Curve25519Scalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f6,
        0x8000000000000000000000000000000,
    ))
        .into();

    assert!(scalar_varint_size(&val) == 37);
}

#[test]
fn scalar_slices_consumes_the_correct_amount_of_bytes() {
    // x = (p - 1)
    let val1 = -Curve25519Scalar::from(1_u64);

    // x = (p + 1) / 2
    let val2: Curve25519Scalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f7,
        0x8000000000000000000000000000000,
    ))
        .into();

    assert!(scalar_varints_size(&[Curve25519Scalar::from(1000_u64)]) == 2);

    assert!(
        scalar_varints_size(&[
            Curve25519Scalar::from(1000_u64),
            Curve25519Scalar::from(0_u64),
            val1,
            Curve25519Scalar::from(2_u64),
            val2
        ]) == 42
    );
}

#[test]
fn small_scalars_are_correctly_encoded_and_decoded_as_positive_varints() {
    let mut buf = [0_u8; 38];

    // x = 0, which is encoded as 2 * 0 = 0
    assert!(write_scalar_varint(&mut buf[..], &Curve25519Scalar::from(0_u64)) == 1);
    assert!(buf[0] == 0);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (Curve25519Scalar::from(0_u64), 1));

    // x = 1, which is encoded as 2 * 1 = 2
    assert!(write_scalar_varint(&mut buf[..], &Curve25519Scalar::from(1_u64)) == 1);
    assert!(buf[0] == 2);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (Curve25519Scalar::from(1_u64), 1));

    // x = 2, which is encoded as 2 * x = 4
    assert!(write_scalar_varint(&mut buf[..], &Curve25519Scalar::from(2_u64)) == 1);
    assert!(buf[0] == 4);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (Curve25519Scalar::from(2_u64), 1));
}

#[test]
fn big_scalars_with_small_additive_inverses_are_correctly_encoded_and_decoded_as_negative_varints()
{
    let mut buf = [0_u8; 38];

    // x = p - 1 (p is the ristretto group order)
    // y = -x = 1
    // which is encoded as -y, or as 2 * y - 1 = 1
    let val = -Curve25519Scalar::from(1u64);
    assert!(write_scalar_varint(&mut buf[..], &val) == 1);
    assert!(buf[0] == 1);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (val, 1));

    // x = p - 2 (p is the ristretto group order)
    // y = -x = 2
    // which is encoded as -y, or as 2 * y - 1 = 3
    let val = -Curve25519Scalar::from(2u64);
    assert!(write_scalar_varint(&mut buf[..], &val) == 1);
    assert!(buf[0] == 3);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (val, 1));
}

#[test]
fn big_scalars_that_are_smaller_than_their_additive_inverses_are_correctly_encoded_and_decoded_as_positive_varints(
) {
    let mut buf = [0_u8; 38];

    // (p - 1) / 2 (p is the ristretto group order)
    // y = -x = (p + 1) / 2 (which is bigger than x)
    let val: Curve25519Scalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f6,
        0x8000000000000000000000000000000,
    ))
        .into();
    assert!(write_scalar_varint(&mut buf[..], &val) == 37);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (val, 37));

    // using a smaller buffer will fail
    assert!((read_scalar_varint(&buf[..10]) as Option<(Curve25519Scalar, _)>).is_none());
}

#[test]
fn big_additive_inverses_that_are_smaller_than_the_input_scalars_are_correctly_encoded_and_decoded_as_negative_varints(
) {
    let mut buf = [0_u8; 38];

    // x = (p + 1) / 2 (p is the group order)
    // y = -x = (p - 1) / 2 (which is smaller than x)
    let val: Curve25519Scalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f7,
        0x8000000000000000000000000000000,
    ))
        .into();

    assert!(write_scalar_varint(&mut buf[..], &val) == 37);
    assert!(read_scalar_varint(&buf[..]).unwrap() == (val, 37));

    // using a smaller buffer will fail
    assert!((read_scalar_varint(&buf[..10]) as Option<(Curve25519Scalar, _)>).is_none());
}

#[test]
fn valid_varint_encoded_input_that_map_to_curve25519_scalars_smaller_than_the_p_field_order_in_the_read_scalar_will_not_wrap_around_p(
) {
    let mut buf = [0b11111111_u8; 36];

    // 252 bits set is fine (252 bits = 36 * 7 as
    //  each byte can hold only 7 bits in the varint encoding)
    buf[35] = 0b01111111_u8;

    // buf represents the number 2^252 - 1
    // removing the varint encoding, we would have y = ((2^252 - 1) // 2 + 1) % p
    // since we want x, we would have x = -y
    let expected_x = -Curve25519Scalar::from(&U256::from_words(
        0x00000000000000000000000000000000,
        0x8000000000000000000000000000000,
    ));

    assert!(read_scalar_varint(&buf[..]).unwrap() == (expected_x, 36));
}

#[test]
fn valid_varint_encoded_input_that_map_to_curve25519_scalars_bigger_than_the_p_field_order_in_the_read_scalar_will_wrap_around_p(
) {
    let mut buf = [0b11111111_u8; 37];

    // we set the first bit to 0 so that we have a positive varint encoding
    buf[0] = 0b11111110;
    // we set the last byte to 31, so that we have 256 bits set, and the MST equal 0
    buf[36] = 0b00001111; // buf has 256 bit-length

    // at this point, buf represents the number 2^256 - 2,
    // which has 256 bit-length, where 255 bits are set to 1
    // also, `expected_val` is simply x = ((2^256 - 2) >> 1) % p
    let expected_val: Curve25519Scalar = (&U256::from_words(
        0x6de72ae98b3ab623977f4a4775473484,
        0xfffffffffffffffffffffffffffffff,
    ))
        .into();
    assert!(read_scalar_varint(&buf[..]).unwrap() == (expected_val, 37));

    // even though we are able to read varint numbers of up to 259 bits-length,
    // we can only represent a number up to 256 bits-length. Bits 257 to 259 are ignored
    buf[36] = 0b00011111; // buf has 257 bit-length
    assert!(read_scalar_varint(&buf[..]).unwrap() == (expected_val, 37));

    buf[36] = 0b00111111; // buf has 258 bit-length
    assert!(read_scalar_varint(&buf[..]).unwrap() == (expected_val, 37));

    buf[36] = 0b01111111; // buf has 259 bit-length
    assert!(read_scalar_varint(&buf[..]).unwrap() == (expected_val, 37));
}

#[test]
fn varint_encoded_values_that_never_ends_will_make_the_read_scalar_to_error_out() {
    let buf = [0b11111111_u8; 5];

    // varint numbers that do not terminate will fail out
    assert!((read_scalar_varint(&buf[..]) as Option<(Curve25519Scalar, _)>).is_none());
}

#[test]
fn valid_varint_encoded_input_that_has_length_bigger_than_259_bits_will_make_the_read_scalar_to_error_out(
) {
    let mut buf = [0b11111111_u8; 38];

    // a varint with 260 bit-length will fail (260 bits = 37 * 7 + 1 as
    //  each byte can hold only 7 bits in the varint encoding)
    buf[37] = 0b00000001_u8;
    assert!((read_scalar_varint(&buf[..37]) as Option<(Curve25519Scalar, _)>).is_none());

    // a varint with 266 bit-length will fail (266 bits = 38 * 7 as
    //  each byte can hold only 7 bits in the varint encoding)
    buf[37] = 0b01111111_u8;
    assert!((read_scalar_varint(&buf[..38]) as Option<(Curve25519Scalar, _)>).is_none());
}

fn write_read_and_compare_encoding(expected_scals: &[Curve25519Scalar]) {
    let mut buf_vec = vec![0_u8; 37 * expected_scals.len()];
    let total_bytes_read = write_scalar_varints(&mut buf_vec[..], expected_scals);

    let buf = &buf_vec[0..total_bytes_read];
    let mut scals =
        vec![Curve25519Scalar::from_le_bytes_mod_order(&[0_u8; 32]); expected_scals.len()];
    read_scalar_varints(&mut scals[..], buf).unwrap();

    for (scal, expected_scal) in scals.iter().zip(expected_scals.iter()) {
        assert_eq!(*scal, *expected_scal);
    }
}

#[test]
fn scalar_slices_are_correctly_encoded_and_decoded() {
    write_read_and_compare_encoding(&[Curve25519Scalar::from(0_u128)]);
    write_read_and_compare_encoding(&[
        Curve25519Scalar::from(1_u64),
        Curve25519Scalar::from(4_u32),
    ]);
    write_read_and_compare_encoding(&[
        Curve25519Scalar::from(1_u64),
        Curve25519Scalar::from(u128::MAX),
        Curve25519Scalar::from(0_u128),
        Curve25519Scalar::from(5_u16),
        Curve25519Scalar::from(u128::MAX),
    ]);

    // x = p - 1 (where p is the ristretto group_order)
    let val = -Curve25519Scalar::from(1_u64);

    write_read_and_compare_encoding(&[
        Curve25519Scalar::from(u128::MAX),
        Curve25519Scalar::from(0_u64),
        val,
        Curve25519Scalar::from(5_u16),
        Curve25519Scalar::from(1_u64),
        Curve25519Scalar::from(0_u64),
        Curve25519Scalar::from(u128::MAX),
    ]);

    // some random scalar
    let bytes: [u8; 32] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0x00,
    ];

    write_read_and_compare_encoding(&[
        Curve25519Scalar::from(u128::MAX),
        Curve25519Scalar::from(0_u64),
        Curve25519Scalar::from_le_bytes_mod_order(&bytes),
        Curve25519Scalar::from(5_u16),
        Curve25519Scalar::from_le_bytes_mod_order(&bytes),
        Curve25519Scalar::from(1_u64),
        Curve25519Scalar::from(0_u64),
        Curve25519Scalar::from(u128::MAX),
    ]);

    // some random scalar
    let bytes: [u8; 32] = [
        0xec, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde,
        0x14, 0xec, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9,
        0xde, 0x00,
    ];

    write_read_and_compare_encoding(&[
        Curve25519Scalar::from(u128::MAX),
        Curve25519Scalar::from(0_u64),
        Curve25519Scalar::from_le_bytes_mod_order(&bytes),
        Curve25519Scalar::from(5_u16),
        Curve25519Scalar::from_le_bytes_mod_order(&bytes),
        Curve25519Scalar::from(1_u64),
        Curve25519Scalar::from(0_u64),
        Curve25519Scalar::from(u128::MAX),
    ]);
}
