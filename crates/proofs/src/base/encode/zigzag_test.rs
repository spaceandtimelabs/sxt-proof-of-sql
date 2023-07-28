use crate::base::encode::ZigZag;
use crate::base::encode::U256;
use crate::base::scalar::ArkScalar;

#[test]
fn small_scalars_are_encoded_as_positive_zigzag_values() {
    // x = 0
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(ArkScalar::from(0_u64).zigzag() == U256::from_words(0, 0));

    // x = 1
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(ArkScalar::from(1_u8).zigzag() == U256::from_words(2, 0));

    // x = 2
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(ArkScalar::from(2_u32).zigzag() == U256::from_words(4, 0));

    // x = u128::MAX
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(
        ArkScalar::from(u128::MAX).zigzag()
            == U256::from_words(0xfffffffffffffffffffffffffffffffe, 0x1)
    );

    for x in 1..1000_u128 {
        // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
        assert!(ArkScalar::from(x).zigzag() == U256::from_words(2 * x, 0));
    }
}

#[test]
fn big_scalars_with_small_additive_inverses_are_encoded_as_negative_zigzag_values() {
    // x = p - 1 (p = 2^252 + 27742317777372353535851937790883648493 is the ristretto group order)
    // the additive inverse of x is y = 1. Since y < x, the ZigZag encodes -y, which is
    // encoded as 2 * y - 1 = 1
    assert!((-ArkScalar::from(1_u32)).zigzag() == U256::from_words(1, 0));

    // x = p - 2 (p = 2^252 + 27742317777372353535851937790883648493 is the ristretto group order)
    // the additive inverse of x is y = 2. Since y < x, the ZigZag encodes -y, which is
    // encoded as 2 * y - 1 = 3
    assert!((-ArkScalar::from(2_u32)).zigzag() == U256::from_words(3, 0));

    for y in 1..1000_u128 {
        // since x > y, where x + y = 0, the ZigZag value is encoded as 2 * y - 1
        assert!((-ArkScalar::from(y)).zigzag() == U256::from_words(2 * y - 1, 0));
    }
}

#[test]
fn big_scalars_that_are_smaller_than_their_additive_inverses_are_encoded_as_positive_zigzag_values()
{
    // x = (p - 1) / 2 (p is the ristretto group order)
    let val: ArkScalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f6,
        0x8000000000000000000000000000000,
    ))
        .into();
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(
        val.zigzag()
            == U256::from_words(
                27742317777372353535851937790883648492,
                21267647932558653966460912964485513216
            )
    );
}

#[test]
fn big_additive_inverses_that_are_smaller_than_the_input_scalars_are_encoded_as_negative_zigzag_values(
) {
    // x = (p + 1) / 2 (p is the ristretto group order)
    let val: ArkScalar = (&U256::from_words(
        0xa6f7cef517bce6b2c09318d2e7ae9f7,
        0x8000000000000000000000000000000,
    ))
        .into();

    // the additive inverse of x is y = -x = (p - 1) / 2
    // since we have y < x, the ZigZag encoding is 2 * y - 1 = p - 2
    assert!(
        val.zigzag()
            == U256::from_words(
                27742317777372353535851937790883648491,
                21267647932558653966460912964485513216
            )
    );

    // x = - U256 { low: 0, high: 0x1_u128 }
    // since x > y, where x + y = 0, the ZigZag value is encoded as 2 * y - 1
    let val: ArkScalar = (&U256 {
        low: 0x0_u128,
        high: 0x1_u128,
    })
        .into();
    assert!((-val).zigzag() == U256::from_words(0xffffffffffffffffffffffffffffffff_u128, 0x1_u128));
}
