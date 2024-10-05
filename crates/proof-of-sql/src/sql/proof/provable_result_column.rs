use super::Indexes;
use crate::{
    base::{database::Column, scalar::Scalar},
    sql::proof::ProvableResultElement,
};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, selection: &Indexes) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize;
}

impl<'a, T: ProvableResultElement<'a>> ProvableResultColumn for &[T] {
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

impl<S: Scalar> ProvableResultColumn for Column<'_, S> {
    fn num_bytes(&self, selection: &Indexes) -> usize {
        match self {
            Column::Boolean(col) => col.num_bytes(selection),
            Column::TinyInt(col) => col.num_bytes(selection),
            Column::SmallInt(col) => col.num_bytes(selection),
            Column::Int(col) => col.num_bytes(selection),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.num_bytes(selection),
            Column::Int128(col) => col.num_bytes(selection),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.num_bytes(selection),
            Column::VarChar((col, _)) => col.num_bytes(selection),
        }
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        match self {
            Column::Boolean(col) => col.write(out, selection),
            Column::TinyInt(col) => col.write(out, selection),
            Column::SmallInt(col) => col.write(out, selection),
            Column::Int(col) => col.write(out, selection),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.write(out, selection),
            Column::Int128(col) => col.write(out, selection),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.write(out, selection),
            Column::VarChar((col, _)) => col.write(out, selection),
        }
    }
}

impl<'a, T: ProvableResultElement<'a>, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self, selection: &Indexes) -> usize {
        (&self[..]).num_bytes(selection)
    }

    fn write(&self, out: &mut [u8], selection: &Indexes) -> usize {
        (&self[..]).write(out, selection)
    }
}
