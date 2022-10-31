use crate::sql::proof::{IntermediateResultColumn, QueryError, QueryResult};
use arrow::array::{Array, Int32Array, Int64Array};
use arrow::datatypes::DataType;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use integer_encoding::VarInt;
use merlin::Transcript;
use std::sync::Arc;

fn read_column<T: VarInt + std::fmt::Display>(
    data: &[u8],
    n: usize,
) -> Result<(Vec<T>, usize), QueryError> {
    let mut res = Vec::with_capacity(n);
    let mut cnt = 0;
    for _ in 0..n {
        let (val, num_read) = match <T>::decode_var(&data[cnt..]) {
            Some(x) => x,
            _ => return Err(QueryError::Overflow),
        };
        res.push(val);
        cnt += num_read;
    }
    Ok((res, cnt))
}

/// An intermediate form of a query result that can be transformed
/// to either the finalized query result form or a query error
pub struct IntermediateQueryResult {
    num_columns: u64,
    indexes: Vec<u64>,
    data: Vec<u8>,
}

impl IntermediateQueryResult {
    /// Form intermediate query result from index rows and result columns
    pub fn new<'a>(
        indexes: &'a [u64],
        columns: &'a [Box<dyn IntermediateResultColumn + 'a>],
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
        IntermediateQueryResult {
            num_columns: columns.len() as u64,
            indexes: indexes.to_vec(),
            data,
        }
    }

    /// Append intermediate query result to transcript
    #[allow(unused_variables)]
    pub fn commit(transcript: &mut Transcript) {
        todo!();
    }

    /// Given an evaluation vector, compute the evaluation of the intermediate result
    /// columns as spare multilinear extensions
    #[allow(unused_variables)]
    pub fn evaluate(evaluation_vec: &[Scalar]) -> Option<Vec<Scalar>> {
        todo!();
    }

    /// Convert the intermediate query result into a final query result
    pub fn into_query_result(&self, schema: SchemaRef) -> QueryResult {
        assert_eq!(schema.fields().len() as u64, self.num_columns);
        let n = self.indexes.len();
        let mut offset: usize = 0;
        let mut columns: Vec<Arc<dyn Array>> = Vec::with_capacity(self.num_columns as usize);
        for field in schema.fields() {
            offset += match field.data_type() {
                DataType::Int64 => {
                    let (col, num_read) = read_column::<i64>(&self.data[offset..], n)?;
                    columns.push(Arc::new(Int64Array::from(col)));
                    Ok(num_read)
                }
                DataType::Int32 => {
                    let (col, num_read) = read_column::<i32>(&self.data[offset..], n)?;
                    columns.push(Arc::new(Int32Array::from(col)));
                    Ok(num_read)
                }
                _ => panic!("unsupported data type"),
            }?;
        }
        assert_eq!(offset, self.data.len());
        Ok(RecordBatch::try_new(schema, columns).unwrap())
    }
}
