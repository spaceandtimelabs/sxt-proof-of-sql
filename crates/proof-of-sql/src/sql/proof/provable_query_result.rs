use super::{decode_and_convert, decode_multiple_elements, ProvableResultColumn, QueryError};
use crate::{
    base::{
        database::{Column, ColumnField, ColumnType, OwnedColumn, OwnedTable, Table},
        polynomial::compute_evaluation_vector,
        scalar::{Scalar, ScalarExt},
    },
    sql::proof::result_element_serialization::decode_fixedsizebinary_elements,
};
use alloc::{vec, vec::Vec};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

/// An intermediate form of a query result that can be transformed
/// to either the finalized query result form or a query error
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProvableQueryResult {
    num_columns: u64,
    pub(crate) table_length: u64,
    data: Vec<u8>,
}

// TODO: Handle truncation properly. The `allow(clippy::cast_possible_truncation)` is a temporary fix and should be replaced with proper logic to manage possible truncation scenarios.
impl ProvableQueryResult {
    #[allow(clippy::cast_possible_truncation)]
    /// The number of columns in the result
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.num_columns as usize
    }
    /// A mutable reference to the number of columns in the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn num_columns_mut(&mut self) -> &mut u64 {
        &mut self.num_columns
    }

    #[allow(clippy::cast_possible_truncation)]
    /// The number of rows in the result
    #[must_use]
    pub fn table_length(&self) -> usize {
        self.table_length as usize
    }
    /// A mutable reference to the underlying encoded data of the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    /// This function is available to allow for easy creation for testing.
    #[cfg(test)]
    #[must_use]
    pub fn new_from_raw_data(num_columns: u64, table_length: u64, data: Vec<u8>) -> Self {
        Self {
            num_columns,
            table_length,
            data,
        }
    }

    /// Form intermediate query result from index rows and result columns
    /// # Panics
    ///
    /// Will panic if `table_length` is somehow larger than the length of some column
    /// which should never happen.
    #[must_use]
    pub fn new<'a, S: Scalar>(table_length: u64, columns: &'a [Column<'a, S>]) -> Self {
        assert!(columns
            .iter()
            .all(|column| table_length == column.len() as u64));
        let mut sz = 0;
        for col in columns {
            sz += col.num_bytes(table_length);
        }
        let mut data = vec![0u8; sz];
        let mut sz = 0;
        for col in columns {
            sz += col.write(&mut data[sz..], table_length);
        }
        ProvableQueryResult {
            num_columns: columns.len() as u64,
            table_length,
            data,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(
        clippy::missing_panics_doc,
        reason = "Assertions ensure preconditions are met, eliminating the possibility of panic."
    )]
    /// Given an evaluation vector, compute the evaluation of the intermediate result
    /// columns as spare multilinear extensions
    ///
    /// # Panics
    /// This function will panic if the length of `evaluation_point` does not match `self.num_columns`.
    /// It will also panic if the `data` array is not properly formatted for the expected column types.
    pub fn evaluate<S: Scalar>(
        &self,
        evaluation_point: &[S],
        output_length: usize,
        column_result_fields: &[ColumnField],
    ) -> Result<Vec<S>, QueryError> {
        if self.num_columns as usize != column_result_fields.len() {
            return Err(QueryError::InvalidColumnCount);
        }
        let mut evaluation_vec = vec![Zero::zero(); output_length];
        compute_evaluation_vector(&mut evaluation_vec, evaluation_point);
        let mut offset: usize = 0;
        let mut res = Vec::with_capacity(self.num_columns as usize);

        for field in column_result_fields {
            let mut val = S::zero();
            for entry in evaluation_vec.iter().take(output_length) {
                let (x, sz) = match field.data_type() {
                    ColumnType::Boolean => decode_and_convert::<bool, S>(&self.data[offset..]),
                    ColumnType::Uint8 => decode_and_convert::<u8, S>(&self.data[offset..]),
                    ColumnType::TinyInt => decode_and_convert::<i8, S>(&self.data[offset..]),
                    ColumnType::SmallInt => decode_and_convert::<i16, S>(&self.data[offset..]),
                    ColumnType::Int => decode_and_convert::<i32, S>(&self.data[offset..]),
                    ColumnType::BigInt => decode_and_convert::<i64, S>(&self.data[offset..]),
                    ColumnType::Int128 => decode_and_convert::<i128, S>(&self.data[offset..]),
                    ColumnType::Decimal75(_, _) | ColumnType::Scalar => {
                        decode_and_convert::<S, S>(&self.data[offset..])
                    }

                    ColumnType::VarChar => decode_and_convert::<&str, S>(&self.data[offset..]),
                    ColumnType::VarBinary => {
                        let (raw_bytes, used) =
                            decode_and_convert::<&[u8], &[u8]>(&self.data[offset..])?;
                        let x = S::from_byte_slice_via_hash(raw_bytes);
                        Ok((x, used))
                    }
                    ColumnType::TimestampTZ(_, _) => {
                        decode_and_convert::<i64, S>(&self.data[offset..])
                    }
                    // need custom decode and convert with custom byte width
                    ColumnType::FixedSizeBinary(_) => {
                        let (raw_bytes, used) =
                            decode_and_convert::<&[u8], &[u8]>(&self.data[offset..])?;
                        let x = S::from_byte_slice_via_hash(raw_bytes);
                        Ok((x, used))
                    }
                }?;
                val += *entry * x;
                offset += sz;
            }
            res.push(val);
        }
        if offset != self.data.len() {
            return Err(QueryError::MiscellaneousEvaluationError);
        }

        Ok(res)
    }

    #[allow(
        clippy::missing_panics_doc,
        reason = "Assertions ensure preconditions are met, eliminating the possibility of panic."
    )]
    /// Convert the intermediate query result into a final query result
    ///
    /// The result is essentially an `OwnedTable` type.
    pub fn to_owned_table<S: Scalar>(
        &self,
        column_result_fields: &[ColumnField],
    ) -> Result<OwnedTable<S>, QueryError> {
        if column_result_fields.len() != self.num_columns() {
            return Err(QueryError::InvalidColumnCount);
        }

        let n = self.table_length();
        let mut offset: usize = 0;

        let owned_table = OwnedTable::try_new(
            column_result_fields
                .iter()
                .map(|field| match field.data_type() {
                    ColumnType::Boolean => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Boolean(col)))
                    }
                    ColumnType::Uint8 => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Uint8(col)))
                    }
                    ColumnType::TinyInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::TinyInt(col)))
                    }
                    ColumnType::SmallInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::SmallInt(col)))
                    }
                    ColumnType::Int => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Int(col)))
                    }
                    ColumnType::BigInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::BigInt(col)))
                    }
                    ColumnType::Int128 => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Int128(col)))
                    }
                    ColumnType::VarChar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::VarChar(col)))
                    }
                    ColumnType::VarBinary => {
                        // Manually specify the item type: `&[u8]`
                        let (decoded_slices, num_read) =
                            decode_multiple_elements::<&[u8]>(&self.data[offset..], n)?;
                        offset += num_read;

                        // Convert those slices to owned `Vec<u8>`
                        let col_vec = decoded_slices.into_iter().map(<[u8]>::to_vec).collect();

                        Ok((field.name(), OwnedColumn::VarBinary(col_vec)))
                    }
                    ColumnType::Scalar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Scalar(col)))
                    }
                    ColumnType::Decimal75(precision, scale) => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Decimal75(precision, scale, col)))
                    }
                    ColumnType::TimestampTZ(tu, tz) => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::TimestampTZ(tu, tz, col)))
                    }
                    ColumnType::FixedSizeBinary(byte_width) => {
                        let bw: usize = byte_width.into();
                        let (col_data, used_bytes) =
                            // try reusing decode multiple elements with n * bw
                            decode_fixedsizebinary_elements(&self.data[offset..], n, bw)?;
                        offset += used_bytes;
                        Ok((
                            field.name(),
                            OwnedColumn::FixedSizeBinary(byte_width, col_data),
                        ))
                    }
                })
                .collect::<Result<_, QueryError>>()?,
        )?;

        assert_eq!(offset, self.data.len());
        assert_eq!(owned_table.num_columns(), self.num_columns());

        Ok(owned_table)
    }
}

impl<S: Scalar> From<Table<'_, S>> for ProvableQueryResult {
    fn from(table: Table<S>) -> Self {
        let num_rows = table.num_rows();
        let columns = table
            .into_inner()
            .into_iter()
            .map(|(_, col)| col)
            .collect::<Vec<_>>();
        Self::new(num_rows as u64, &columns)
    }
}
