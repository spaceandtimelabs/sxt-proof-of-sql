use curve25519_dalek::scalar::Scalar;

/// U256 represents an unsigned 256-bits integer number
///
/// low is the lower bytes of the u256 number (from 0 to 127 bits)
/// high is the upper bytes of the u256 number (from 128 to 255 bits)
#[derive(PartialEq, Eq)]
pub struct U256 {
    pub low: u128,
    pub high: u128,
}

impl U256 {
    #[inline]
    pub const fn from_words(low: u128, high: u128) -> Self {
        U256 { low, high }
    }
}

/// This trait converts a dalek scalar into a U256 integer
impl From<&Scalar> for U256 {
    fn from(val: &Scalar) -> Self {
        let bytes = val.as_bytes();

        let low = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        let high = u128::from_le_bytes(bytes[16..32].try_into().unwrap());

        U256::from_words(low, high)
    }
}

/// This trait converts a U256 integer into a dalek scalar
impl From<&U256> for Scalar {
    fn from(val: &U256) -> Self {
        let bytes_low = val.low.to_le_bytes();
        let bytes_high = val.high.to_le_bytes();
        let bytes: [u8; 32] = [bytes_low, bytes_high].concat().try_into().unwrap();

        Scalar::from_bytes_mod_order(bytes)
    }
}
