//! This module provides utility functions for working with Arrow schemas in the context of Proof of SQL.
//! It includes functionality to convert Arrow schemas to PoSQL-compatible formats.

use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

/// Converts an Arrow schema to a PoSQL-compatible schema.
///
/// This function takes an Arrow `SchemaRef` and returns a new `SchemaRef` where
/// floating-point data types (Float16, Float32, Float64) are converted to Decimal256(75, 30).
/// Other data types remain unchanged.
///
/// # Arguments
///
/// * `schema` - The input Arrow schema to convert.
///
/// # Returns
///
/// A new `SchemaRef` with PoSQL-compatible data types.
#[must_use]
pub fn get_posql_compatible_schema(schema: &SchemaRef) -> SchemaRef {
    let new_fields: Vec<Field> = schema
        .fields()
        .iter()
        .map(|field| {
            let new_data_type = match field.data_type() {
                DataType::Float16 | DataType::Float32 | DataType::Float64 => {
                    DataType::Decimal256(20, 10)
                }
                _ => field.data_type().clone(),
            };
            Field::new(field.name(), new_data_type, field.is_nullable())
        })
        .collect();

    Arc::new(Schema::new(new_fields))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_float_fields_to_decimal_and_preserves_other_metadata() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("f16", DataType::Float16, true),
            Field::new("f32", DataType::Float32, false),
            Field::new("f64", DataType::Float64, true),
            Field::new("name", DataType::Utf8, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);
        let fields = converted.fields();

        assert_eq!(fields[0].data_type(), &DataType::Decimal256(20, 10));
        assert!(fields[0].is_nullable());
        assert_eq!(fields[1].data_type(), &DataType::Decimal256(20, 10));
        assert!(!fields[1].is_nullable());
        assert_eq!(fields[2].data_type(), &DataType::Decimal256(20, 10));
        assert!(fields[2].is_nullable());
        assert_eq!(fields[3].data_type(), &DataType::Utf8);
        assert!(!fields[3].is_nullable());
    }
}
