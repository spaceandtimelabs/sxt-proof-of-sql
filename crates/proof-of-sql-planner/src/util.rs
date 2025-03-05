use super::{PlannerError, PlannerResult};
use arrow::datatypes::{Field, Schema};
use datafusion::{
    catalog::TableReference,
    common::{Column, DFSchema, ScalarValue},
};
use proof_of_sql::base::database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef};

/// Convert a [`TableReference`] to a [`TableRef`]
///
/// If catalog is provided it is ignored
pub(crate) fn table_reference_to_table_ref(table: &TableReference) -> TableRef {
    match table {
        TableReference::Bare { table } => TableRef::from_names(None, table),
        TableReference::Partial { schema, table } | TableReference::Full { schema, table, .. } => {
            TableRef::from_names(Some(schema), table)
        }
    }
}

/// Convert a [`ScalarValue`] to a [`LiteralValue`]
///
/// TODO: add other types supported in `PoSQL`
#[allow(dead_code)]
pub(crate) fn scalar_value_to_literal_value(value: &ScalarValue) -> PlannerResult<LiteralValue> {
    match value {
        ScalarValue::Boolean(Some(v)) => Ok(LiteralValue::Boolean(*v)),
        ScalarValue::Int8(Some(v)) => Ok(LiteralValue::TinyInt(*v)),
        ScalarValue::Int16(Some(v)) => Ok(LiteralValue::SmallInt(*v)),
        ScalarValue::Int32(Some(v)) => Ok(LiteralValue::Int(*v)),
        ScalarValue::Int64(Some(v)) => Ok(LiteralValue::BigInt(*v)),
        ScalarValue::UInt8(Some(v)) => Ok(LiteralValue::Uint8(*v)),
        ScalarValue::Utf8(Some(v)) => Ok(LiteralValue::VarChar(v.clone())),
        _ => Err(PlannerError::UnsupportedDataType {
            data_type: value.data_type().clone(),
        }),
    }
}

/// Find a column in a schema and return its info as a [`ColumnRef`]
///
/// Note that the table name must be provided in the column which resolved logical plans do
/// Otherwise we error out
#[allow(dead_code)]
pub(crate) fn column_to_column_ref(column: &Column, schema: &DFSchema) -> PlannerResult<ColumnRef> {
    let relation = column
        .relation
        .as_ref()
        .ok_or_else(|| PlannerError::UnresolvedLogicalPlan)?;
    let field = schema.field_with_name(Some(relation), &column.name)?;
    let table_ref = table_reference_to_table_ref(relation);
    let column_type = ColumnType::try_from(field.data_type().clone()).map_err(|_e| {
        PlannerError::UnsupportedDataType {
            data_type: field.data_type().clone(),
        }
    })?;
    Ok(ColumnRef::new(
        table_ref,
        column.name.as_str().into(),
        column_type,
    ))
}

/// Convert a Vec<ColumnField> to a Schema
pub(crate) fn column_fields_to_schema(column_fields: Vec<ColumnField>) -> Schema {
    Schema::new(
        column_fields
            .into_iter()
            .map(|column_field| {
                //TODO: Make columns nullable
                let data_type = (&column_field.data_type()).into();
                Field::new(column_field.name().value.as_str(), data_type, false)
            })
            .collect::<Vec<_>>(),
    )
}

#[allow(clippy::missing_panics_doc)]
#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;

    // TableReference to TableRef
    #[test]
    fn we_can_convert_table_reference_to_table_ref() {
        // Bare
        let table = TableReference::bare("table");
        assert_eq!(
            table_reference_to_table_ref(&table),
            TableRef::from_names(None, "table")
        );

        // Partial
        let table = TableReference::partial("schema", "table");
        assert_eq!(
            table_reference_to_table_ref(&table),
            TableRef::from_names(Some("schema"), "table")
        );

        // Full
        let table = TableReference::full("catalog", "schema", "table");
        assert_eq!(
            table_reference_to_table_ref(&table),
            TableRef::from_names(Some("schema"), "table")
        );
    }

    // ScalarValue to LiteralValue
    #[test]
    fn we_can_convert_scalar_value_to_literal_value() {
        // Boolean
        let value = ScalarValue::Boolean(Some(true));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::Boolean(true)
        );

        // Int8
        let value = ScalarValue::Int8(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::TinyInt(1)
        );

        // Int16
        let value = ScalarValue::Int16(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::SmallInt(1)
        );

        // Int32
        let value = ScalarValue::Int32(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::Int(1)
        );

        // Int64
        let value = ScalarValue::Int64(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::BigInt(1)
        );

        // UInt8
        let value = ScalarValue::UInt8(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::Uint8(1)
        );

        // Utf8
        let value = ScalarValue::Utf8(Some("value".to_string()));
        assert_eq!(
            scalar_value_to_literal_value(&value).unwrap(),
            LiteralValue::VarChar("value".to_string())
        );

        // Unsupported
        let value = ScalarValue::Float32(Some(1.0));
        assert!(matches!(
            scalar_value_to_literal_value(&value),
            Err(PlannerError::UnsupportedDataType { .. })
        ));
    }

    // Column to ColumnRef
    #[test]
    fn we_can_convert_column_to_column_ref() {
        let column = Column::new(Some("namespace.table"), "a");
        let arrow_schema = Schema::new(vec![Field::new("a", DataType::Int32, false)]);
        let df_schema =
            DFSchema::try_from_qualified_schema("namespace.table", &arrow_schema).unwrap();
        assert_eq!(
            column_to_column_ref(&column, &df_schema).unwrap(),
            ColumnRef::new(
                TableRef::from_names(Some("namespace"), "table"),
                "a".into(),
                ColumnType::Int
            )
        );
    }

    #[test]
    fn we_cannot_convert_column_to_column_ref_without_relation() {
        let column = Column::new(None::<&str>, "a");
        let arrow_schema = Schema::new(vec![Field::new("a", DataType::Int32, false)]);
        let df_schema = DFSchema::try_from(arrow_schema).unwrap();
        assert!(matches!(
            column_to_column_ref(&column, &df_schema),
            Err(PlannerError::UnresolvedLogicalPlan)
        ));
    }

    #[test]
    fn we_cannot_convert_column_to_column_ref_with_invalid_column_name() {
        let column = Column::new(Some("namespace.table"), "b");
        let arrow_schema = Schema::new(vec![Field::new("a", DataType::Int32, false)]);
        let df_schema =
            DFSchema::try_from_qualified_schema("namespace.table", &arrow_schema).unwrap();
        assert!(matches!(
            column_to_column_ref(&column, &df_schema),
            Err(PlannerError::DataFusionError { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_column_to_column_ref_with_unsupported_data_type() {
        let column = Column::new(Some("namespace.table"), "a");
        let arrow_schema = Schema::new(vec![Field::new("a", DataType::Float32, false)]);
        let df_schema =
            DFSchema::try_from_qualified_schema("namespace.table", &arrow_schema).unwrap();
        assert!(matches!(
            column_to_column_ref(&column, &df_schema),
            Err(PlannerError::UnsupportedDataType { .. })
        ));
    }

    // ColumnFields to Schema
    #[test]
    fn we_can_convert_column_fields_to_schema() {
        // Empty
        let column_fields = vec![];
        let schema = column_fields_to_schema(column_fields);
        assert_eq!(schema.all_fields(), Vec::<&Field>::new());

        // Non-empty
        let column_fields = vec![
            ColumnField::new("a".into(), ColumnType::SmallInt),
            ColumnField::new("b".into(), ColumnType::VarChar),
        ];
        let schema = column_fields_to_schema(column_fields);
        assert_eq!(
            schema.all_fields(),
            vec![
                &Field::new("a", DataType::Int16, false),
                &Field::new("b", DataType::Utf8, false),
            ]
        );
    }
}
