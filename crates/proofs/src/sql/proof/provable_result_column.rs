use super::Indexes;
use crate::{base::database::Column, sql::proof::EncodeProvableResultElement};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, selection: &Indexes) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize;
}

impl<'a, T: EncodeProvableResultElement> ProvableResultColumn for &'a [T] {
    fn num_bytes(&self, selection: &Indexes) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self[i as usize].required_bytes();
        }
        res
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        let mut res = 0;
        for i in selection.iter() {
            res += self[i as usize].encode(&mut out[res..]);
        }
        res
    }
}

impl ProvableResultColumn for Column<'_> {
    fn num_bytes(&self, selection: &Indexes) -> usize {
        match self {
            Column::BigInt(col) => col.num_bytes(selection),
            Column::Int128(col) => col.num_bytes(selection),
            Column::VarChar((col, _)) => col.num_bytes(selection),
            #[cfg(test)]
            Column::Scalar(col) => col.num_bytes(selection),
        }
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        match self {
            Column::BigInt(col) => col.write(out, selection),
            Column::Int128(col) => col.write(out, selection),
            Column::VarChar((col, _)) => col.write(out, selection),
            #[cfg(test)]
            Column::Scalar(col) => col.write(out, selection),
        }
    }
}

impl<T: EncodeProvableResultElement, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self, selection: &Indexes) -> usize {
        (&self[..]).num_bytes(selection)
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        (&self[..]).write(out, selection)
    }
}
