use crate::{
    base::{database::Column, scalar::Scalar},
    sql::proof::ProvableResultElement,
};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, length: u64) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], length: u64) -> usize;
}

impl<'a, T: ProvableResultElement<'a>> ProvableResultColumn for &[T] {
    fn num_bytes(&self, length: u64) -> usize {
        assert_eq!(self.len() as u64, length);
        self.iter().map(ProvableResultElement::required_bytes).sum()
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        let mut res = 0;
        for i in 0..length {
            res += self[i as usize].encode(&mut out[res..]);
        }
        res
    }
}

impl<S: Scalar> ProvableResultColumn for Column<'_, S> {
    fn num_bytes(&self, length: u64) -> usize {
        match self {
            Column::Boolean(_, col) => col.num_bytes(length),
            Column::TinyInt(_, col) => col.num_bytes(length),
            Column::SmallInt(_, col) => col.num_bytes(length),
            Column::Int(_, col) => col.num_bytes(length),
            Column::BigInt(_, col) | Column::TimestampTZ(.., col) => col.num_bytes(length),
            Column::Int128(_, col) => col.num_bytes(length),
            Column::Decimal75(_, _, _, col) | Column::Scalar(_, col) => col.num_bytes(length),
            Column::VarChar(_, (col, _)) => col.num_bytes(length),
        }
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        match self {
            Column::Boolean(_, col) => col.write(out, length),
            Column::TinyInt(_, col) => col.write(out, length),
            Column::SmallInt(_, col) => col.write(out, length),
            Column::Int(_, col) => col.write(out, length),
            Column::BigInt(_, col) | Column::TimestampTZ(.., col) => col.write(out, length),
            Column::Int128(_, col) => col.write(out, length),
            Column::Decimal75(.., col) | Column::Scalar(_, col) => col.write(out, length),
            Column::VarChar(_, (col, _)) => col.write(out, length),
        }
    }
}

impl<'a, T: ProvableResultElement<'a>, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self, length: u64) -> usize {
        (&self[..]).num_bytes(length)
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        (&self[..]).write(out, length)
    }
}
