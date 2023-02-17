use crate::base::database::{Column, ColumnType};
use crate::base::scalar::compute_commitment_for_testing;
use crate::base::scalar::ToScalar;
use indexmap::IndexMap;
use proofs_sql::Identifier;

use arrow::array::Array;
use arrow::array::Int64Array;
use arrow::array::StringArray;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use std::sync::Arc;

/// This TestAccessorColumn is an owned version for the Column defined values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestAccessorColumn {
    BigInt(Vec<i64>),
    VarChar((Vec<String>, Vec<Scalar>)),
}

/// This TestAccessorColumns is a simple alias for multiple accessors
pub type TestAccessorColumns = IndexMap<Identifier, TestAccessorColumn>;

impl TestAccessorColumn {
    /// Map the TestAccessorColumn type to some accessor ColumnType
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
        }
    }

    /// Compute the commitment value associated with the accessor column
    pub fn compute_commitment(&self, offset: usize) -> RistrettoPoint {
        match self {
            Self::BigInt(v) => compute_commitment_for_testing(&v[..], offset),
            Self::VarChar((_v, s)) => compute_commitment_for_testing(&s[..], offset),
        }
    }

    /// Map the TestAccessorColumn values to some accessor Column
    pub fn to_column<'a>(&'a self, column_type: ColumnType, alloc: &'a Bump) -> Column {
        match self {
            Self::BigInt(v) => {
                assert_eq!(ColumnType::BigInt, column_type);
                Column::BigInt(&v[..])
            }
            Self::VarChar((v, s)) => {
                assert_eq!(ColumnType::VarChar, column_type);

                let v = alloc.alloc_slice_fill_with(v.len(), |i| -> &'a [u8] { v[i].as_bytes() });
                Column::HashedBytes((v, &s[..]))
            }
        }
    }

    /// Map the TestAccessorColumn values to some arrow array
    pub fn to_arrow(&self) -> Arc<dyn Array> {
        match self {
            Self::BigInt(v) => Arc::new(Int64Array::from(v.to_vec())),
            Self::VarChar((v, _s)) => {
                let v: Vec<_> = v.iter().map(|v| v.as_str()).collect();
                Arc::new(StringArray::from(v))
            }
        }
    }
}

/// Convert a polars tuple to a TestAccessorColumn
///
/// Note that the series value must live longer than the TestAccessor
/// since it may reference data from it. For instance, see `TestAccessorColumn::HashBytes`.
impl From<(&polars::prelude::Field, &polars::prelude::Series)> for TestAccessorColumn {
    fn from(
        field_series: (&polars::prelude::Field, &polars::prelude::Series),
    ) -> TestAccessorColumn {
        let (field, series) = field_series;

        match field.data_type() {
            polars::datatypes::DataType::UInt8
            | polars::datatypes::DataType::UInt16
            | polars::datatypes::DataType::UInt32
            | polars::datatypes::DataType::UInt64
            | polars::datatypes::DataType::Int8
            | polars::datatypes::DataType::Int16
            | polars::datatypes::DataType::Int32
            | polars::datatypes::DataType::Int64 => {
                let col_rows = series.cast(&polars::datatypes::DataType::Int64).unwrap();
                let col = col_rows.i64().unwrap().cont_slice().unwrap();

                TestAccessorColumn::BigInt(col.to_vec())
            }
            polars::datatypes::DataType::Utf8 => {
                let col: Vec<_> = series
                    .utf8()
                    .unwrap()
                    .into_iter()
                    .map(|opt_v| opt_v.unwrap().to_string())
                    .collect();
                let col_scalars: Vec<_> = col.iter().map(|v| v.as_str().to_scalar()).collect();

                TestAccessorColumn::VarChar((col, col_scalars))
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::database::TableRef;

    use super::*;

    #[test]
    fn we_can_get_the_correct_column_type_with_big_int() {
        let t = TestAccessorColumn::BigInt(vec![123]);
        assert_eq!(t.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_get_the_correct_column_type_with_varchar() {
        let data = vec!["abc".to_string()];
        let v2 = data.iter().map(|v| (&v[..]).to_scalar()).collect();
        let t = TestAccessorColumn::VarChar((data, v2));
        assert_eq!(t.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn we_can_get_the_correct_column_with_big_int() {
        let data = vec![123];
        let alloc = Bump::new();
        let t = TestAccessorColumn::BigInt(data.clone());
        assert_eq!(
            t.to_column(ColumnType::BigInt, &alloc),
            Column::BigInt(&data[..])
        );
    }

    #[test]
    fn we_can_get_the_correct_column_with_varchar() {
        let data = vec!["abc".to_string()];
        let alloc = Bump::new();
        let s: Vec<_> = data.iter().map(|v| (&v[..]).to_scalar()).collect();
        let t = TestAccessorColumn::VarChar((data.clone(), s.clone()));
        let data_slice: Vec<_> = data.iter().map(|v| v.as_bytes()).collect();
        assert_eq!(
            t.to_column(ColumnType::VarChar, &alloc),
            Column::HashedBytes((&data_slice[..], &s[..]))
        );
    }

    #[test]
    fn we_can_get_the_correct_arrow_array_with_big_int() {
        let data = vec![123];
        let t = TestAccessorColumn::BigInt(data.clone());
        assert_eq!(*t.to_arrow(), Int64Array::from(data));
    }

    #[test]
    fn we_can_get_the_correct_arrow_array_with_varchar() {
        let data = vec!["abc".to_string()];
        let v2 = data.iter().map(|v| (&v[..]).to_scalar()).collect();
        let t = TestAccessorColumn::VarChar((data.clone(), v2));
        assert_eq!(*t.to_arrow(), StringArray::from(data));
    }

    #[test]
    fn we_can_get_the_correct_commitment_with_big_int() {
        let data = vec![123];
        let t = TestAccessorColumn::BigInt(data.clone());
        let offset = 0;
        assert_eq!(
            t.compute_commitment(offset),
            compute_commitment_for_testing(&data[..], offset)
        );
        let offset = 11;
        assert_eq!(
            t.compute_commitment(offset),
            compute_commitment_for_testing(&data[..], offset)
        );
    }

    #[test]
    fn we_can_get_the_correct_commitment_with_varchar() {
        let data = vec!["abc"];
        let v2 = data.iter().map(|v| v.to_scalar()).collect();
        let t = TestAccessorColumn::VarChar((data.iter().map(|v| v.to_string()).collect(), v2));
        let offset = 0;
        assert_eq!(
            t.compute_commitment(offset),
            compute_commitment_for_testing(&data[..], offset)
        );
        let offset = 11;
        assert_eq!(
            t.compute_commitment(offset),
            compute_commitment_for_testing(&data[..], offset)
        );
    }

    #[test]
    fn serializing_tableref_to_json() {
        let table_ref: TableRef = "databasename.tablename".parse().unwrap();
        let json = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(json, r#""databasename.tablename""#);
    }

    #[test]
    fn deserializing_tableref_from_json() {
        let table_ref: TableRef = "databasename.tablename".parse().unwrap();
        let json = r#""databasename.tablename""#;
        let deserialized: TableRef = serde_json::from_str(json).unwrap();
        assert_eq!(table_ref, deserialized);
    }

    #[test]
    fn deserializing_tableref_fails_for_invalid_identifier() {
        let json = r#""databasename.table name""#;
        let deserialized: Result<TableRef, _> = serde_json::from_str(json);
        assert!(deserialized.is_err());
    }
}
