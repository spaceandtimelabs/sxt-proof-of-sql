use super::QueryError;
use super::{
    are_indexes_valid, decode_multiple_elements, DecodeProvableResultElement, ProvableResultColumn,
    QueryResult,
};
use crate::base::database::{ColumnField, ColumnType};

use crate::base::polynomial::Scalar;
use arrow::array::{Array, Int64Array, StringArray};
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An intermediate form of a query result that can be transformed
/// to either the finalized query result form or a query error
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ProvableQueryResult {
    pub num_columns: u64,
    pub indexes: Vec<u64>,
    pub data: Vec<u8>,
}

impl ProvableQueryResult {
    /// Form intermediate query result from index rows and result columns
    #[tracing::instrument(
        name = "proofs.sql.proof.provable_query_result.new",
        level = "debug",
        skip_all
    )]
    pub fn new<'a>(indexes: &'a [u64], columns: &'a [Box<dyn ProvableResultColumn + 'a>]) -> Self {
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
            indexes: indexes.to_vec(),
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
    pub fn evaluate(
        &self,
        evaluation_vec: &[Scalar],
        column_result_fields: &[ColumnField],
    ) -> Option<Vec<Scalar>> {
        assert_eq!(self.num_columns as usize, column_result_fields.len());

        if !are_indexes_valid(&self.indexes, evaluation_vec.len()) {
            return None;
        }

        let mut offset: usize = 0;
        let mut res = Vec::with_capacity(self.num_columns as usize);

        for field in column_result_fields {
            let mut val = Scalar::zero();
            for index in self.indexes.iter() {
                let (x, sz) = match field.data_type() {
                    ColumnType::BigInt => <i64>::decode_to_scalar(&self.data[offset..]),
                    ColumnType::VarChar => <&str>::decode_to_scalar(&self.data[offset..]),
                }?;

                val += evaluation_vec[*index as usize] * x;
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
    #[tracing::instrument(
        name = "proofs.sql.proof.provable_query_result.into_query_result",
        level = "debug",
        skip_all
    )]
    pub fn into_query_result(&self, column_result_fields: &[ColumnField]) -> QueryResult {
        assert_eq!(column_result_fields.len() as u64, self.num_columns);

        let n = self.indexes.len();
        let mut offset: usize = 0;
        let mut column_fields: Vec<_> = Vec::with_capacity(self.num_columns as usize);
        let mut columns: Vec<Arc<dyn Array>> = Vec::with_capacity(self.num_columns as usize);

        for field in column_result_fields {
            offset += match field.data_type() {
                ColumnType::BigInt => {
                    let (col, num_read) = decode_multiple_elements::<i64>(&self.data[offset..], n)
                        .ok_or(QueryError::Overflow)?;

                    columns.push(Arc::new(Int64Array::from(col)));

                    Ok(num_read)
                }
                ColumnType::VarChar => {
                    let (col, num_read) = decode_multiple_elements::<&str>(&self.data[offset..], n)
                        .ok_or(QueryError::InvalidString)?;

                    columns.push(Arc::new(StringArray::from(col)));

                    Ok(num_read)
                }
            }?;

            column_fields.push(field.into());
        }

        assert_eq!(offset, self.data.len());

        let schema = Arc::new(Schema::new(column_fields));

        Ok(RecordBatch::try_new(schema, columns).unwrap())
    }
}
