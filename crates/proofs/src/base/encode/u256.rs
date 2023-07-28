use crate::base::scalar::ArkScalar;

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
impl From<&ArkScalar> for U256 {
    fn from(val: &ArkScalar) -> Self {
        let buf: [u64; 4] = val.into();
        let low: u128 = (buf[0] as u128) | (buf[1] as u128) << 64;
        let high: u128 = (buf[2] as u128) | (buf[3] as u128) << 64;
        U256::from_words(low, high)
    }
}

/// This trait converts a U256 integer into a dalek scalar
impl From<&U256> for ArkScalar {
    fn from(val: &U256) -> Self {
        let bytes = [val.low.to_le_bytes(), val.high.to_le_bytes()].concat();
        ArkScalar::from_le_bytes_mod_order(&bytes)
    }
}
