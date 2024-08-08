use ark_bls12_381::Fr;
use zerocopy::AsBytes;

pub trait OffsetToBytes {
    const IS_SIGNED: bool;
    fn byte_size() -> usize;
    fn min_as_fr() -> Fr;
    fn offset_to_bytes(&self) -> Vec<u8>;
}

impl OffsetToBytes for u8 {
    const IS_SIGNED: bool = false;

    fn byte_size() -> usize {
        std::mem::size_of::<u8>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl OffsetToBytes for i16 {
    const IS_SIGNED: bool = true;

    fn byte_size() -> usize {
        std::mem::size_of::<i16>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(i16::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i16::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i32 {
    const IS_SIGNED: bool = true;

    fn byte_size() -> usize {
        std::mem::size_of::<i32>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(i32::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i32::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i64 {
    const IS_SIGNED: bool = true;

    fn byte_size() -> usize {
        std::mem::size_of::<i64>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(i64::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i64::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i128 {
    const IS_SIGNED: bool = true;

    fn byte_size() -> usize {
        std::mem::size_of::<i128>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(i128::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i128::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for bool {
    const IS_SIGNED: bool = false;

    fn byte_size() -> usize {
        std::mem::size_of::<bool>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(false)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl OffsetToBytes for u64 {
    const IS_SIGNED: bool = false;

    fn byte_size() -> usize {
        std::mem::size_of::<u64>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes();
        bytes.to_vec()
    }
}

impl OffsetToBytes for [u64; 4] {
    const IS_SIGNED: bool = false;

    fn byte_size() -> usize {
        std::mem::size_of::<[u64; 4]>()
    }

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let slice = self.as_bytes();
        slice.to_vec()
    }
}
