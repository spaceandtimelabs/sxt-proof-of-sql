use num_traits::{identities::One, int::PrimInt};
use std::ops::{Neg, Sub};
/// BitDecompose is a trait that converts a nonnegative number into it's bits. This should match with the bits of the `Scalar` when implementing `IntoScalar`.
pub trait BitDecompose {
    /// The bits give be the binary representation of a nonnegative `self`. The behaviour is undefined for negative values. This should match with the bits of the embedding into `Scalar` when implementing `IntoScalar`.
    fn bits(&self) -> Vec<bool>;
}
/// This is a helper trait for `PositiveProof` that allows for converting signed numeric values to their bits in a compact way.
pub trait SignedBitDecompose: BitDecompose + Neg<Output = Self> + Clone {
    /// Gives the bits of `self` minus 1. Only should be run on positive values.
    fn sub_one_bits(&self) -> Vec<bool> {
        self.sub_one().bits()
    }
    /// Gives the bits of the negative of `self`. Only should be run on nonpositive values.
    fn neg_bits(&self) -> Vec<bool> {
        self.clone().neg().bits()
    }
    /// Gives the value of `self` minus 1.
    fn sub_one(&self) -> Self;
}
impl<T> SignedBitDecompose for T
where
    T: Sub<Output = T> + Neg<Output = T> + BitDecompose + One + Clone,
{
    fn sub_one(&self) -> Self {
        self.clone().sub(Self::one())
    }
}

impl<T> BitDecompose for T
where
    T: PrimInt,
{
    fn bits(&self) -> Vec<bool> {
        (0..(Self::zero().count_zeros() - self.leading_zeros()))
            .map(|i| self.shr(i as usize).bitand(Self::one()) == (Self::one()))
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use crate::base::math::{BitDecompose, SignedBitDecompose};

    #[test]
    fn test_bit_decomposition() {
        assert_eq!(0b1011_u8.bits(), vec![true, true, false, true]);
        assert_eq!((-0b1011_i8).neg_bits(), vec![true, true, false, true]);
        assert_eq!(0b1011_i8.sub_one_bits(), vec![false, true, false, true]);
        assert_eq!(0_u8.bits(), vec![false; 0]);
        assert_eq!(0_i8.neg_bits(), vec![false; 0]);
        assert_eq!(0b1011_u128.bits(), vec![true, true, false, true]);
        assert_eq!((-0b1011_i128).neg_bits(), vec![true, true, false, true]);
        assert_eq!(0b1011_i128.sub_one_bits(), vec![false, true, false, true]);
        assert_eq!(0_u128.bits(), vec![false; 0]);
        assert_eq!(0_i128.neg_bits(), vec![false; 0]);
    }
}
