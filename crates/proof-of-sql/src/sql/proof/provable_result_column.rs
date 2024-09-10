use crate::{
    base::{database::Column, scalar::Scalar},
    sql::proof::ProvableResultElement,
};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8]) -> usize;
}

impl<'a, T: ProvableResultElement<'a>> ProvableResultColumn for &[T] {
    fn num_bytes(&self) -> usize {
        self.iter().map(|x| x.required_bytes()).sum()
    }

    fn write(&self, out: &mut [u8]) -> usize {
        let mut res = 0;
        for val in self.iter() {
            res += val.encode(&mut out[res..]);
        }
        res
    }
}

impl<S: Scalar> ProvableResultColumn for Column<'_, S> {
    fn num_bytes(&self) -> usize {
        match self {
            Column::Boolean(col) => col.num_bytes(),
            Column::SmallInt(col) => col.num_bytes(),
            Column::Int(col) => col.num_bytes(),
            Column::BigInt(col) => col.num_bytes(),
            Column::Int128(col) => col.num_bytes(),
            Column::Decimal75(_, _, col) => col.num_bytes(),
            Column::Scalar(col) => col.num_bytes(),
            Column::VarChar((col, _)) => col.num_bytes(),
            Column::TimestampTZ(_, _, col) => col.num_bytes(),
        }
    }

    fn write(&self, out: &mut [u8]) -> usize {
        match self {
            Column::Boolean(col) => col.write(out),
            Column::SmallInt(col) => col.write(out),
            Column::Int(col) => col.write(out),
            Column::BigInt(col) => col.write(out),
            Column::Int128(col) => col.write(out),
            Column::Decimal75(_, _, col) => col.write(out),
            Column::Scalar(col) => col.write(out),
            Column::VarChar((col, _)) => col.write(out),
            Column::TimestampTZ(_, _, col) => col.write(out),
        }
    }
}

impl<'a, T: ProvableResultElement<'a>, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self) -> usize {
        (&self[..]).num_bytes()
    }

    fn write(&self, out: &mut [u8]) -> usize {
        (&self[..]).write(out)
    }
}
