mod u256;
pub(crate) use u256::U256;

mod zigzag;
pub(crate) use zigzag::ZigZag;

#[cfg(test)]
mod zigzag_test;

mod scalar_varint;

#[cfg(test)]
mod scalar_varint_test;

mod varint_trait;
pub use varint_trait::VarInt;
#[cfg(test)]
mod varint_trait_test;
