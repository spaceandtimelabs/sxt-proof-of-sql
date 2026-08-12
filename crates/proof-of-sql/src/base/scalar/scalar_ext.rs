/// Explicit limb conversions for scalar field elements.
///
/// These methods replace opaque `From`/`Into`/`RefInto<[u64; 4]>` bounds so call
/// sites opt into little-endian non-Montgomery limb mapping deliberately.
///
/// Implementors must reduce `from_limbs` modulo the field order.
pub trait ScalarExt: Sized {
    /// Converts a little-endian limb array into a scalar, reducing modulo the field order.
    fn from_limbs(val: [u64; 4]) -> Self;

    /// Converts a scalar into its canonical little-endian non-Montgomery limb representation.
    fn to_limbs(&self) -> [u64; 4];
}
