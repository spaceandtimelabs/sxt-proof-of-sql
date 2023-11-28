use super::Indexes;
use crate::{base::database::Column, sql::proof::EncodeProvableResultElement};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, selection: &Indexes) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize;
}

/// Support using a database column as a result in-place
pub struct DenseProvableResultColumn<'a, T: EncodeProvableResultElement> {
    data: &'a [T],
}

impl<'a, T: EncodeProvableResultElement> DenseProvableResultColumn<'a, T> {
    /// Form result column from a slice of its values
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }
}

impl<'a, T: EncodeProvableResultElement> ProvableResultColumn for DenseProvableResultColumn<'a, T>
where
    [T]: ToOwned,
{
    fn num_bytes(&self, selection: &Indexes) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self.data[i as usize].required_bytes();
        }
        res
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self.data[i as usize].encode(&mut out[res..]);
        }
        res
    }
}

impl<'a> From<Column<'a>> for Box<dyn ProvableResultColumn + 'a> {
    fn from(col: Column<'a>) -> Self {
        match col {
            Column::BigInt(col) => Box::new(DenseProvableResultColumn::new(col)),
            Column::Int128(col) => Box::new(DenseProvableResultColumn::new(col)),
            Column::VarChar((col, _)) => Box::new(DenseProvableResultColumn::new(col)),
            #[cfg(test)]
            Column::Scalar(col) => Box::new(DenseProvableResultColumn::new(col)),
        }
    }
}
