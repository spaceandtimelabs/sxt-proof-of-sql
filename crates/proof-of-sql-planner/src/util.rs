use super::{PlannerError, PlannerResult};
use arrow::datatypes::{Field, Schema};
use datafusion::{
    catalog::TableReference,
    common::{Column, DFSchema, ScalarValue},
    logical_expr::expr::Placeholder,
};
use proof_of_sql::{
    base::{
        database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    sql::proof_exprs::DynProofExpr,
};
use sqlparser::ast::Ident;

/// Parse a placeholder string of the form "$1", "$2", etc. into a `usize`.
fn parse_placeholder_id(s: &str) -> Option<usize> {
    s.strip_prefix('$')
        // Must be all digits
        .filter(|digits| digits.chars().all(|c| c.is_ascii_digit()) && !digits.starts_with('0'))
        // Finally, parse
        .and_then(|digits| digits.parse().ok())
}

/// Convert a datafusion [`Placeholder`] to a Proof of SQL [`PlaceholderExpr`]
#[expect(clippy::missing_panics_doc, reason = "can not actually panic")]
pub(crate) fn placeholder_to_placeholder_expr(
    placeholder: &Placeholder,
) -> PlannerResult<DynProofExpr> {
    let df_id = placeholder.id.clone();
    let df_type = placeholder.data_type.clone();
    let posql_id = parse_placeholder_id(&df_id)
        .ok_or_else(|| PlannerError::InvalidPlaceholderId { id: df_id.clone() })?;
    let posql_type = df_type
        .clone()
        .ok_or(PlannerError::UntypedPlaceholder {
            placeholder: placeholder.clone(),
        })?
        .try_into()
        .map_err(|_| PlannerError::UnsupportedDataType {
            data_type: df_type.clone().unwrap(),
        })?;
    Ok(DynProofExpr::try_new_placeholder(posql_id, posql_type)?)
}

/// Convert a [`TableReference`] to a [`TableRef`]
///
/// If catalog is provided it errors out
pub(crate) fn table_reference_to_table_ref(table: &TableReference) -> PlannerResult<TableRef> {
    match table {
        TableReference::Bare { table } => Ok(TableRef::from_names(None, table)),
        TableReference::Partial { schema, table } => Ok(TableRef::from_names(Some(schema), table)),
        TableReference::Full { .. } => Err(PlannerError::CatalogNotSupported),
    }
}

/// Convert a [`ScalarValue`] to a [`LiteralValue`]
///
/// TODO: add other types supported in `PoSQL`
pub(crate) fn scalar_value_to_literal_value(value: ScalarValue) -> PlannerResult<LiteralValue> {
    match value {
        ScalarValue::Boolean(Some(v)) => Ok(LiteralValue::Boolean(v)),
        ScalarValue::Int8(Some(v)) => Ok(LiteralValue::TinyInt(v)),
        ScalarValue::Int16(Some(v)) => Ok(LiteralValue::SmallInt(v)),
        ScalarValue::Int32(Some(v)) => Ok(LiteralValue::Int(v)),
        ScalarValue::Int64(Some(v)) => Ok(LiteralValue::BigInt(v)),
        ScalarValue::UInt8(Some(v)) => Ok(LiteralValue::Uint8(v)),
        ScalarValue::Utf8(Some(v)) => Ok(LiteralValue::VarChar(v)),
        ScalarValue::Binary(Some(v)) => Ok(LiteralValue::VarBinary(v)),
        ScalarValue::TimestampSecond(Some(v), None) => Ok(LiteralValue::TimeStampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            v,
        )),
        ScalarValue::TimestampMillisecond(Some(v), None) => Ok(LiteralValue::TimeStampTZ(
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            v,
        )),
        ScalarValue::TimestampMicrosecond(Some(v), None) => Ok(LiteralValue::TimeStampTZ(
            PoSQLTimeUnit::Microsecond,
            PoSQLTimeZone::utc(),
            v,
        )),
        ScalarValue::TimestampNanosecond(Some(v), None) => Ok(LiteralValue::TimeStampTZ(
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            v,
        )),
        ScalarValue::Decimal128(Some(v), precision, scale) => {
            Ok(LiteralValue::Decimal75(precision, scale, v.into()))
        }
        ScalarValue::Decimal256(Some(v), precision, scale) => {
            Ok(LiteralValue::Decimal75(precision, scale, v.into()))
        }
        _ => Err(PlannerError::UnsupportedDataType {
            data_type: value.data_type().clone(),
        }),
    }
}

/// Find a column in a schema and return its info as a [`ColumnRef`]
///
/// Note that the table name must be provided in the column which resolved logical plans do
/// Otherwise we error out
pub(crate) fn column_to_column_ref(column: &Column, schema: &DFSchema) -> PlannerResult<ColumnRef> {
    let relation = column
        .relation
        .as_ref()
        .ok_or_else(|| PlannerError::UnresolvedLogicalPlan)?;
    let field = schema.field_with_name(Some(relation), &column.name)?;
    let table_ref = table_reference_to_table_ref(relation)?;
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
#[must_use]
pub fn column_fields_to_schema(column_fields: Vec<ColumnField>) -> Schema {
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

/// Convert a [`DFSchema`] to a Vec<ColumnField>
///
/// Note that this returns an error if any column has an unsupported `DataType`
pub(crate) fn df_schema_to_column_fields(schema: &DFSchema) -> PlannerResult<Vec<ColumnField>> {
    schema
        .fields()
        .iter()
        .map(|field| -> PlannerResult<ColumnField> {
            let column_type = ColumnType::try_from(field.data_type().clone()).map_err(|_e| {
                PlannerError::UnsupportedDataType {
                    data_type: field.data_type().clone(),
                }
            })?;
            Ok(ColumnField::new(Ident::new(field.name()), column_type))
        })
        .collect::<PlannerResult<Vec<ColumnField>>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::DataType;

    // parse_placeholder_id
    #[test]
    fn we_can_parse_valid_placeholder_id() {
        // "$1" => Some(1)
        assert_eq!(parse_placeholder_id("$1"), Some(1));
        // "$123" => Some(123)
        assert_eq!(parse_placeholder_id("$123"), Some(123));
    }

    #[test]
    fn we_cannot_parse_placeholder_id_without_dollar_sign() {
        // "" => None
        assert_eq!(parse_placeholder_id(""), None);
        // "1" => None
        assert_eq!(parse_placeholder_id("1"), None);
    }

    #[test]
    fn we_cannot_parse_placeholder_id_empty_after_dollar_sign() {
        // "$" => None
        assert_eq!(parse_placeholder_id("$"), None);
    }

    #[test]
    fn we_cannot_parse_placeholder_id_with_non_digits() {
        // "$abc" => None
        assert_eq!(parse_placeholder_id("$abc"), None);
        // "$1x" => None
        assert_eq!(parse_placeholder_id("$1x"), None);
    }

    #[test]
    fn we_cannot_parse_placeholder_id_with_leading_zero() {
        // "$0" => None
        assert_eq!(parse_placeholder_id("$0"), None);
        // "$01" => None
        assert_eq!(parse_placeholder_id("$01"), None);
    }

    // placeholder_to_placeholder_expr
    #[test]
    fn we_can_convert_valid_placeholder_to_placeholder_expr() {
        let placeholder = Placeholder {
            id: "$42".to_string(),
            data_type: Some(DataType::Int32),
        };
        let expected = DynProofExpr::try_new_placeholder(42, ColumnType::Int).unwrap();
        let result = placeholder_to_placeholder_expr(&placeholder).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_cannot_convert_placeholder_without_type() {
        let placeholder = Placeholder {
            id: "$1".to_string(),
            data_type: None,
        };
        assert!(matches!(
            placeholder_to_placeholder_expr(&placeholder),
            Err(PlannerError::UntypedPlaceholder { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_placeholder_with_invalid_id() {
        let placeholder = Placeholder {
            // Something invalid like "$0" or "1"
            id: "$0".to_string(),
            data_type: Some(DataType::Int32),
        };
        assert!(matches!(
            placeholder_to_placeholder_expr(&placeholder),
            Err(PlannerError::InvalidPlaceholderId { .. })
        ));
    }

    // TableReference to TableRef
    #[test]
    fn we_can_convert_table_reference_to_table_ref() {
        // Bare
        let table = TableReference::bare("table");
        assert_eq!(
            table_reference_to_table_ref(&table).unwrap(),
            TableRef::from_names(None, "table")
        );

        // Partial
        let table = TableReference::partial("schema", "table");
        assert_eq!(
            table_reference_to_table_ref(&table).unwrap(),
            TableRef::from_names(Some("schema"), "table")
        );
    }

    #[test]
    fn we_cannot_convert_full_table_reference_to_table_ref() {
        let table = TableReference::full("catalog", "schema", "table");
        assert!(matches!(
            table_reference_to_table_ref(&table),
            Err(PlannerError::CatalogNotSupported)
        ));
    }

    // ScalarValue to LiteralValue
    #[test]
    fn we_can_convert_scalar_value_to_literal_value() {
        // Boolean
        let value = ScalarValue::Boolean(Some(true));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Boolean(true)
        );

        // Int8
        let value = ScalarValue::Int8(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::TinyInt(1)
        );

        // Int16
        let value = ScalarValue::Int16(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::SmallInt(1)
        );

        // Int32
        let value = ScalarValue::Int32(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Int(1)
        );

        // Int64
        let value = ScalarValue::Int64(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::BigInt(1)
        );

        // UInt8
        let value = ScalarValue::UInt8(Some(1));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Uint8(1)
        );

        // Utf8
        let value = ScalarValue::Utf8(Some("value".to_string()));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::VarChar("value".to_string())
        );

        // Binary
        let value = ScalarValue::Binary(Some(vec![72, 97, 108, 108, 101, 108, 117, 106, 97, 104]));
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::VarBinary(vec![72, 97, 108, 108, 101, 108, 117, 106, 97, 104])
        );

        // TimestampSecond
        // Thu Mar 06 2025 04:43:12 GMT+0000
        let value = ScalarValue::TimestampSecond(Some(1_741_236_192_i64), None);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                1_741_236_192_i64
            )
        );

        // TimestampMillisecond
        let value = ScalarValue::TimestampMillisecond(Some(1_741_236_192_004_i64), None);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                1_741_236_192_004_i64
            )
        );

        // TimestampMicrosecond
        let value = ScalarValue::TimestampMicrosecond(Some(1_741_236_192_004_000_i64), None);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Microsecond,
                PoSQLTimeZone::utc(),
                1_741_236_192_004_000_i64
            )
        );

        // TimestampNanosecond
        let value = ScalarValue::TimestampNanosecond(Some(1_741_236_192_123_456_789_i64), None);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::TimeStampTZ(
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::utc(),
                1_741_236_192_123_456_789_i64
            )
        );
    }

    #[expect(clippy::cast_sign_loss)]
    #[test]
    fn we_can_convert_scalar_value_to_literal_value_for_decimals() {
        // Decimal128
        let value = ScalarValue::Decimal128(Some(123), 38, 0);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                38_u8,
                0,
                proof_of_sql::base::math::i256::I256::from(123i128)
            )
        );

        // Test edge cases for Decimal128
        let value = ScalarValue::Decimal128(Some(i128::MIN), 38, 10);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                38_u8,
                10,
                proof_of_sql::base::math::i256::I256::from(i128::MIN)
            )
        );

        let value = ScalarValue::Decimal128(Some(i128::MAX), 28, -5);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                28_u8,
                -5,
                proof_of_sql::base::math::i256::I256::from(i128::MAX)
            )
        );

        let value = ScalarValue::Decimal128(Some(0), 38, 0);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(38_u8, 0, proof_of_sql::base::math::i256::I256::from(0i128))
        );

        // Decimal256
        let value = ScalarValue::Decimal256(Some(arrow::datatypes::i256::from_i128(-456)), 75, 120);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                75_u8,
                120,
                proof_of_sql::base::math::i256::I256::from(-456i128)
            )
        );

        // Test edge cases for Decimal256
        let value = ScalarValue::Decimal256(Some(arrow::datatypes::i256::MIN), 75, 127);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                75_u8,
                127,
                proof_of_sql::base::math::i256::I256::new([0, 0, 0, i64::MIN as u64])
            )
        );
        let value = ScalarValue::Decimal256(Some(arrow::datatypes::i256::MAX), 75, -128);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(
                75_u8,
                -128,
                proof_of_sql::base::math::i256::I256::new([
                    u64::MAX,
                    u64::MAX,
                    u64::MAX,
                    i64::MAX as u64
                ])
            )
        );
        let value = ScalarValue::Decimal256(Some(arrow::datatypes::i256::ZERO), 75, 0);
        assert_eq!(
            scalar_value_to_literal_value(value).unwrap(),
            LiteralValue::Decimal75(75_u8, 0, proof_of_sql::base::math::i256::I256::from(0i128))
        );
    }

    #[test]
    fn we_cannot_convert_scalar_value_to_literal_value_if_unsupported() {
        // Unsupported
        let value = ScalarValue::Float32(Some(1.0));
        assert!(matches!(
            scalar_value_to_literal_value(value),
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

    // DFSchema to Vec<ColumnField>
    #[test]
    fn we_can_convert_df_schema_to_column_fields() {
        // Empty
        let arrow_schema = Schema::new(Vec::<Field>::new());
        let df_schema = DFSchema::try_from(arrow_schema).unwrap();
        let column_fields = df_schema_to_column_fields(&df_schema).unwrap();
        assert_eq!(column_fields, Vec::<ColumnField>::new());

        // Non-empty
        let arrow_schema = Schema::new(vec![
            Field::new("a", DataType::Int16, false),
            Field::new("b", DataType::Utf8, false),
        ]);
        let df_schema = DFSchema::try_from(arrow_schema).unwrap();
        let column_fields = df_schema_to_column_fields(&df_schema).unwrap();
        assert_eq!(
            column_fields,
            vec![
                ColumnField::new("a".into(), ColumnType::SmallInt),
                ColumnField::new("b".into(), ColumnType::VarChar),
            ]
        );
    }

    #[test]
    fn we_cannot_convert_df_schema_to_column_fields_with_unsupported_data_type() {
        let arrow_schema = Schema::new(vec![Field::new("a", DataType::Float32, false)]);
        let df_schema = DFSchema::try_from(arrow_schema).unwrap();
        assert!(matches!(
            df_schema_to_column_fields(&df_schema),
            Err(PlannerError::UnsupportedDataType { .. })
        ));
    }
}
