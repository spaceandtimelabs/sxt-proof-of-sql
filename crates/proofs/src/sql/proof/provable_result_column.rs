use integer_encoding::VarInt;

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, selection: &[u64]) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], selection: &[u64]) -> usize;
}

/// Support using a database column as a result in-place
pub struct DenseProvableResultColumn<'a, T: VarInt> {
    data: &'a [T],
}

impl<'a, T: VarInt> DenseProvableResultColumn<'a, T> {
    /// Form result column from a slice of its values
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }
}

impl<'a, T: VarInt> ProvableResultColumn for DenseProvableResultColumn<'a, T> {
    fn num_bytes(&self, selection: &[u64]) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self.data[*i as usize].required_space();
        }
        res
    }

    fn write(&self, out: &mut [u8], selection: &[u64]) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self.data[*i as usize].encode_var(&mut out[res..]);
        }
        res
    }
}
