use super::{
    decode_and_convert, decode_multiple_elements, Indexes, ProvableResultColumn, QueryError,
};
use crate::base::{
    database::{ColumnField, ColumnType, OwnedColumn, OwnedTable},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};

/// An intermediate form of a query result that can be transformed
/// to either the finalized query result form or a query error
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ProvableQueryResult {
    num_columns: u64,
    indexes: Indexes,
    data: Vec<u8>,
}

impl ProvableQueryResult {
    /// The number of columns in the result
    pub fn num_columns(&self) -> usize {
        self.num_columns as usize
    }
    /// The indexes in the result.
    pub fn indexes(&self) -> &Indexes {
        &self.indexes
    }
    /// A mutable reference to a the indexes in the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn indexes_mut(&mut self) -> &mut Indexes {
        &mut self.indexes
    }
    /// A mutable reference to the number of columns in the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn num_columns_mut(&mut self) -> &mut u64 {
        &mut self.num_columns
    }
    /// A mutable reference to the underlying encoded data of the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    /// This function is available to allow for easy creation for testing.
    #[cfg(test)]
    pub fn new_from_raw_data(num_columns: u64, indexes: Indexes, data: Vec<u8>) -> Self {
        Self {
            num_columns,
            indexes,
            data,
        }
    }

    /// Form intermediate query result from index rows and result columns
    #[tracing::instrument(
        name = "proofs.sql.proof.provable_query_result.new",
        level = "debug",
        skip_all
    )]
    pub fn new<'a>(
        indexes: &'a Indexes,
        columns: &'a [Box<dyn ProvableResultColumn + 'a>],
    ) -> Self {
        let mut sz = 0;
        for col in columns.iter() {
            sz += col.num_bytes(indexes);
        }
        let mut data = vec![0u8; sz];
        let mut sz = 0;
        for col in columns.iter() {
            sz += col.write(&mut data[sz..], indexes);
        }
        ProvableQueryResult {
            num_columns: columns.len() as u64,
            indexes: indexes.clone(),
            data,
        }
    }

    /// Given an evaluation vector, compute the evaluation of the intermediate result
    /// columns as spare multilinear extensions
    #[tracing::instrument(
        name = "proofs.sql.proof.provable_query_result.evaluate",
        level = "debug",
        skip_all
    )]
    pub fn evaluate<S: Scalar>(
        &self,
        evaluation_vec: &[S],
        column_result_fields: &[ColumnField],
    ) -> Option<Vec<S>> {
        assert_eq!(self.num_columns as usize, column_result_fields.len());

        if !self.indexes.valid(evaluation_vec.len()) {
            return None;
        }

        let mut offset: usize = 0;
        let mut res = Vec::with_capacity(self.num_columns as usize);

        for field in column_result_fields {
            let mut val = S::zero();
            for index in self.indexes.iter() {
                let (x, sz) = match field.data_type() {
                    ColumnType::Boolean => S::decode_var(&self.data[offset..]),
                    ColumnType::BigInt => S::decode_var(&self.data[offset..]),
                    ColumnType::VarChar => decode_and_convert::<&str, S>(&self.data[offset..]),
                    ColumnType::Int128 => S::decode_var(&self.data[offset..]),
                    ColumnType::Scalar => S::decode_var(&self.data[offset..]),
                    ColumnType::Decimal75(_, _) => S::decode_var(&self.data[offset..]),
                }?;

                val += evaluation_vec[index as usize] * x;
                offset += sz;
            }
            res.push(val);
        }

        if offset != self.data.len() {
            return None;
        }

        Some(res)
    }

    /// Convert the intermediate query result into a final query result
    ///
    /// The result is essentially an `OwnedTable` type.
    #[tracing::instrument(
        name = "proofs.sql.proof.provable_query_result.into_owned_table",
        level = "debug",
        skip_all
    )]
    pub fn into_owned_table<S: Scalar>(
        &self,
        column_result_fields: &[ColumnField],
    ) -> Result<OwnedTable<S>, QueryError> {
        assert_eq!(column_result_fields.len(), self.num_columns());

        let n = self.indexes.len();
        let mut offset: usize = 0;

        let owned_table = OwnedTable::try_new(
            column_result_fields
                .iter()
                .map(|field| match field.data_type() {
                    ColumnType::Boolean => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::Overflow)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Boolean(col)))
                    }
                    ColumnType::BigInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::Overflow)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::BigInt(col)))
                    }
                    ColumnType::Int128 => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::Overflow)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Int128(col)))
                    }
                    ColumnType::VarChar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::InvalidString)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::VarChar(col)))
                    }
                    ColumnType::Scalar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::Overflow)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Scalar(col)))
                    }
                    ColumnType::Decimal75(precision, scale) => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)
                            .ok_or(QueryError::Overflow)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Decimal75(precision, scale, col)))
                    }
                })
                .collect::<Result<_, QueryError>>()?,
        )?;

        assert_eq!(offset, self.data.len());
        assert_eq!(owned_table.num_columns(), self.num_columns());

        Ok(owned_table)
    }
}
