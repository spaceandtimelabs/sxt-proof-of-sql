use crate::base::scalar::Scalar;
use zerocopy::AsBytes;

/// Trait for serializing primitive types in a way that is compatible with efficient proof verification on the EVM.
pub trait PrimitiveSerializeExt<S: Scalar>: Sized {
    /// Serializes a slice of bytes.
    ///
    /// # Arguments
    ///
    /// * `value` - A slice of bytes to be serialized.
    ///
    /// # Returns
    ///
    /// * `Self` - The serialized result.
    fn serialize_slice(self, value: &[u8]) -> Self;

    /// Serializes a single byte.
    ///
    /// # Arguments
    ///
    /// * `value` - A byte to be serialized.
    ///
    /// # Returns
    ///
    /// * `Self` - The serialized result.
    fn serialize_u8(self, value: u8) -> Self {
        self.serialize_slice(&[value])
    }

    /// Serializes a scalar value. The scalar is serialized as a 256-bit, bytewise-big-endian integer.
    /// This is the format used by the EVM for representing integers.
    ///
    /// # Arguments
    ///
    /// * `value` - A scalar value to be serialized.
    ///
    /// # Returns
    ///
    /// * `Self` - The serialized result.
    fn serialize_scalar(self, value: S) -> Self {
        let mut limbs: [u64; 4] = value.into();
        limbs.as_bytes_mut().reverse();
        self.serialize_slice(limbs.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveSerializeExt;
    use crate::base::scalar::{test_scalar::TestScalar, Scalar};
    use core::{iter, marker::PhantomData};
    use itertools::Itertools;
    struct MockSerializer<S: Scalar>(Vec<u8>, PhantomData<S>);
    impl<S: Scalar> MockSerializer<S> {
        fn new() -> Self {
            MockSerializer(Vec::new(), PhantomData)
        }
        fn into_inner(self) -> Vec<u8> {
            self.0
        }
    }
    impl<S: Scalar> PrimitiveSerializeExt<S> for MockSerializer<S> {
        fn serialize_slice(mut self, value: &[u8]) -> Self {
            self.0.extend_from_slice(value);
            self
        }
    }

    #[test]
    fn we_can_serialize_u8() {
        let serializer = MockSerializer::<TestScalar>::new();
        let result = serializer.serialize_u8(123).into_inner();
        assert_eq!(result, vec![123]);
    }

    #[test]
    fn we_can_serialize_scalar_that_requires_one_bytes() {
        let serializer = MockSerializer::<TestScalar>::new();
        let bytes = serializer
            .serialize_scalar(TestScalar::from(123))
            .into_inner();
        assert_eq!(
            bytes,
            iter::empty::<u8>()
                .chain([0; 31])
                .chain([123])
                .collect_vec()
        );
    }

    #[test]
    fn we_can_serialize_scalar_that_requires_two_bytes() {
        let serializer = MockSerializer::<TestScalar>::new();
        let bytes = serializer
            .serialize_scalar(TestScalar::from(123 + (45 << 8)))
            .into_inner();
        assert_eq!(
            bytes,
            iter::empty::<u8>()
                .chain([0; 30])
                .chain([45, 123])
                .collect_vec()
        );
    }
}
