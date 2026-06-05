use arrow::datatypes::{DataType, Field, Schema};
use datafusion::{
    catalog::TableReference,
    common::{Column, DFSchema},
    logical_expr::Expr,
};

/// Create a `Expr::Column` from full table name and column
pub(crate) fn df_column(table_name: &str, column: &str) -> Expr {
    Expr::Column(Column::new(
        Some(TableReference::from(table_name)),
        column.to_string(),
    ))
}

/// Create a `DFSchema` from table name, column name and data type pairs
///
/// Note that nulls are not allowed in the schema
pub(crate) fn df_schema(table_name: &str, pairs: Vec<(&str, DataType)>) -> DFSchema {
    let arrow_schema = Schema::new(
        pairs
            .into_iter()
            .map(|(name, data_type)| Field::new(name, data_type, false))
            .collect::<Vec<_>>(),
    );
    DFSchema::try_from_qualified_schema(table_name, &arrow_schema).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn df_column_qualifies_the_column_with_the_table_name() {
        let expr = df_column("orders", "total");

        match expr {
            Expr::Column(column) => {
                assert_eq!(column.relation, Some(TableReference::from("orders")));
                assert_eq!(column.name, "total");
            }
            other => panic!("expected Expr::Column, got {other:?}"),
        }
    }

    #[test]
    fn df_schema_marks_fields_as_non_nullable_and_qualified() {
        let schema = df_schema(
            "orders",
            vec![("id", DataType::Int64), ("total", DataType::Float64)],
        );

        let fields = schema.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "id");
        assert_eq!(fields[0].data_type(), &DataType::Int64);
        assert!(!fields[0].is_nullable());
        assert_eq!(fields[1].name(), "total");
        assert_eq!(fields[1].data_type(), &DataType::Float64);
        assert!(!fields[1].is_nullable());

        let qualifiers = schema.iter().map(|(q, _)| q.cloned()).collect::<Vec<_>>();
        assert_eq!(
            qualifiers,
            vec![
                Some(TableReference::from("orders")),
                Some(TableReference::from("orders")),
            ]
        );
    }
}
