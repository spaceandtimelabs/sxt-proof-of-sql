use super::{PlannerError, PlannerResult};
use datafusion::{
    catalog::TableReference,
    common::{Column, DFSchema},
    logical_expr::{Expr, LogicalPlan},
    scalar::ScalarValue,
};
use proof_of_sql::base::database::{ColumnRef, ColumnType, LiteralValue, TableRef};

/// Convert a [`TableReference`] to a [`TableRef`]
///
/// If catalog is provided it is ignored
pub(crate) fn table_reference_as_table_ref(table: &TableReference) -> TableRef {
    match table {
        TableReference::Bare { table } => TableRef::from_names(None, &table),
        TableReference::Partial { schema, table } => TableRef::from_names(Some(&schema), &table),
        TableReference::Full { schema, table, .. } => TableRef::from_names(Some(&schema), &table),
    }
}

/// Convert a [`ScalarValue`] to a [`LiteralValue`]
///
/// TODO: add other types supported in PoSQL
pub(crate) fn scalar_value_as_literal_value(value: &ScalarValue) -> PlannerResult<LiteralValue> {
    match value {
        ScalarValue::Boolean(Some(v)) => Ok(LiteralValue::Boolean(*v)),
        ScalarValue::Int8(Some(v)) => Ok(LiteralValue::TinyInt(*v)),
        ScalarValue::Int16(Some(v)) => Ok(LiteralValue::SmallInt(*v)),
        ScalarValue::Int32(Some(v)) => Ok(LiteralValue::Int(*v)),
        ScalarValue::Int64(Some(v)) => Ok(LiteralValue::BigInt(*v)),
        ScalarValue::UInt8(Some(v)) => Ok(LiteralValue::Uint8(*v)),
        _ => Err(PlannerError::InternalError {
            message: "Resolved logical plans should not contain null values",
        }),
    }
}

/// Find a column in a schema and return its info as a [`ColumnRef`]
///
/// Note that the table name must be provided in the column which resolved logical plans do
/// Otherwise we error out
pub(crate) fn column_as_column_ref(column: &Column, schema: &DFSchema) -> PlannerResult<ColumnRef> {
    let relation = column
        .relation
        .as_ref()
        .ok_or(PlannerError::InternalError {
            message: "Resolved logical plans should not contain columns without tables",
        })?;
    let field = schema.field_with_name(Some(relation), &column.name)?;
    let table_ref = table_reference_as_table_ref(relation);
    let column_type =
        ColumnType::try_from(*field.data_type()).map_err(|_e| PlannerError::InternalError {
            message: "Resolved logical plans should not contain columns with unsupported types",
        })?;
    Ok(ColumnRef::new(
        table_ref,
        column.name.as_str().into(),
        column_type,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;

    // TableReference to TableRef
    #[test]
    fn we_can_do_table_reference_as_table_ref() {
        // Bare
        let table = TableReference::bare("table");
        assert_eq!(
            table_reference_as_table_ref(&table),
            TableRef::from_names(None, "table")
        );

        // Partial
        let table = TableReference::partial("schema", "table");
        assert_eq!(
            table_reference_as_table_ref(&table),
            TableRef::from_names(Some("schema"), "table")
        );

        // Full
        let table = TableReference::full("catalog", "schema", "table");
        assert_eq!(
            table_reference_as_table_ref(&table),
            TableRef::from_names(Some("schema"), "table")
        );
    }
}
