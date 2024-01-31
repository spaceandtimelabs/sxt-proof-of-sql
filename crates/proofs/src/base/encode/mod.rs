mod u256;
pub(crate) use u256::U256;

mod zigzag;
pub(crate) use zigzag::ZigZag;

#[cfg(test)]
mod zigzag_test;

mod varint;
pub use varint::{
    read_scalar_varint, read_scalar_varints, scalar_varint_size, scalar_varints_size,
    write_scalar_varint, write_scalar_varints,
};

#[cfg(test)]
mod varint_test;

mod varint_trait;
pub use varint_trait::VarInt;
#[cfg(test)]
mod varint_trait_test;
