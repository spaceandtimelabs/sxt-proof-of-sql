use core::{mem, slice};
use curve25519_dalek::scalar::Scalar;

pub fn as_byte_slice(slice: &[Scalar]) -> &[u8] {
    let slice = slice;
    let len = slice.len() * mem::size_of::<Scalar>();
    unsafe { slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}
