use crate::{
    intermediate_ast::{OrderBy as IntermediateOrderBy, OrderByDirection, Slice},
    Identifier, ResourceId,
};
use alloc::{string::ToString, vec};
use sqlparser::ast::{Expr, Ident, ObjectName, Offset, OffsetRows, OrderByExpr, Value};

/// Converts a [`Identifier`] from the `PoSQL` AST to an [`Ident`] for the `SQLParser` AST.
impl From<Identifier> for Ident {
    fn from(id: Identifier) -> Self {
        Ident::new(id.as_str())
    }
}

/// Converts a [`ResourceId`] from the `PoSQL` AST to an [`ObjectName`] for the `SQLParser` AST.
impl From<ResourceId> for ObjectName {
    fn from(resource_id: ResourceId) -> Self {
        let schema_ident = Ident::new(resource_id.schema().as_str());
        let object_name_ident = Ident::new(resource_id.object_name().as_str());
        ObjectName(vec![schema_ident, object_name_ident])
    }
}

/// Converts an [`IntermediateOrderBy`] from the intermediate AST to a [`OrderByExpr`] for the `SQLParser` AST.
impl From<IntermediateOrderBy> for OrderByExpr {
    fn from(intermediate_order_by: IntermediateOrderBy) -> Self {
        // Convert Identifier to Expr
        let expr = Expr::Identifier(intermediate_order_by.expr.into());

        let asc = match intermediate_order_by.direction {
            OrderByDirection::Asc => Some(true),
            OrderByDirection::Desc => Some(false),
        };

        // Create the OrderByExpr
        OrderByExpr {
            expr,
            asc,
            nulls_first: None,
        }
    }
}

/// Converts a [`Slice`] representing pagination into an [`Offset`] for the `SQLParser`.
impl From<Slice> for Offset {
    fn from(slice: Slice) -> Self {
        let value_expr = Expr::Value(Value::Number(slice.offset_value.to_string(), false));

        let rows = match slice.number_rows {
            u64::MAX => OffsetRows::None, // No specific row offset
            1 => OffsetRows::Row,         // For a single row offset
            _ => OffsetRows::Rows,        // For multiple rows
        };

        Offset {
            value: value_expr,
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_conversion() {
        let test_cases = vec![
            ("ValidIdentifier", "valididentifier"),
            ("INVALID$IDENTIFIER", "invalid$identifier"),
            ("MiXeDCaSeIdentifier", "mixedcaseidentifier"),
            ("", ""),
        ];

        for (input, expected) in test_cases {
            let identifier = Identifier::new(input);
            let ident: Ident = identifier.into();
            assert_eq!(ident.value, expected);
        }
    }

    #[test]
    fn test_conversion_order_by_asc() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(true));
        assert_eq!(
            order_by_expr.expr,
            Expr::Identifier(Ident::new("column_name"))
        );
    }

    #[test]
    fn test_conversion_order_by_desc() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Desc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(false));
        assert_eq!(
            order_by_expr.expr,
            Expr::Identifier(Ident::new("column_name"))
        );
    }

    #[test]
    fn test_conversion_order_by_nulls_first() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new("column_name"),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.nulls_first, None);
    }

    #[test]
    fn test_edge_case_empty_order_by() {
        let intermediate_order_by = IntermediateOrderBy {
            expr: Identifier::new(""),
            direction: OrderByDirection::Asc,
        };

        let order_by_expr: OrderByExpr = OrderByExpr::from(intermediate_order_by);

        assert_eq!(order_by_expr.asc, Some(true));
        assert_eq!(order_by_expr.expr, Expr::Identifier(Ident::new("")));
    }

    #[test]
    fn test_slice_to_offset_conversion_all_rows() {
        let slice = Slice {
            number_rows: u64::MAX,
            offset_value: 0,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("0".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::None);
    }

    #[test]
    fn test_slice_to_offset_conversion_single_row() {
        let slice = Slice {
            number_rows: 1,
            offset_value: 5,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("5".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Row);
    }

    #[test]
    fn test_slice_to_offset_conversion_multiple_rows() {
        let slice = Slice {
            number_rows: 10,
            offset_value: -2,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("-2".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Rows);
    }

    #[test]
    fn test_slice_to_offset_conversion_zero_offset() {
        let slice = Slice {
            number_rows: 10,
            offset_value: 0,
        };
        let offset: Offset = slice.into();

        assert_eq!(
            offset.value,
            Expr::Value(Value::Number("0".to_string(), false))
        );
        assert_eq!(offset.rows, OffsetRows::Rows);
    }

    #[test]
    fn test_resource_id_to_object_name_conversion() {
        let resource_id =
            ResourceId::new(Identifier::new("my_schema"), Identifier::new("my_table"));

        let object_name: ObjectName = resource_id.into();

        assert_eq!(object_name.0.len(), 2); // Should have two parts
        assert_eq!(object_name.0[0].value, "my_schema");
        assert_eq!(object_name.0[1].value, "my_table");
    }
}
