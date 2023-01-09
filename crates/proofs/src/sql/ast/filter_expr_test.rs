use crate::base::database::{ColumnRef, ColumnType};
use crate::sql::ast::EqualsExpr;
use crate::sql::ast::FilterExpr;
use crate::sql::ast::FilterResultExpr;
use crate::sql::ast::TableExpr;
use crate::sql::ast::{AndExpr, NotExpr, OrExpr};
use crate::sql::proof::QueryExpr;
use std::collections::HashSet;

use arrow::datatypes::Field;
use arrow::datatypes::Schema;
use curve25519_dalek::scalar::Scalar;
use std::sync::Arc;

#[test]
fn we_can_correctly_fetch_the_query_result_schema() {
    let provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(
                ColumnRef {
                    column_name: "a".to_string(),
                    table_name: "sxt_tab".to_string(),
                    schema: None,
                    column_type: ColumnType::BigInt,
                },
                "a".to_string(),
            ),
            FilterResultExpr::new(
                ColumnRef {
                    column_name: "b".to_string(),
                    table_name: "sxt_tab".to_string(),
                    schema: None,
                    column_type: ColumnType::BigInt,
                },
                "b".to_string(),
            ),
        ],
        TableExpr {
            name: "sxt_tab".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "c".to_string(),
                table_name: "sxt_tab".to_string(),
                schema: None,
                column_type: ColumnType::BigInt,
            },
            Scalar::from(123_u64),
        )),
    );

    let result_schema = provable_ast.get_result_schema();

    assert_eq!(
        result_schema,
        Arc::new(Schema::new(vec![
            Field::new("a", (&ColumnType::BigInt).into(), false,),
            Field::new("b", (&ColumnType::BigInt).into(), false,)
        ]))
    );
}

#[test]
fn we_can_correctly_fetch_all_the_referenced_columns() {
    let provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(
                ColumnRef {
                    column_name: "a".to_string(),
                    table_name: "sxt_tab".to_string(),
                    schema: None,
                    column_type: ColumnType::BigInt,
                },
                "a".to_string(),
            ),
            FilterResultExpr::new(
                ColumnRef {
                    column_name: "f".to_string(),
                    table_name: "sxt_tab".to_string(),
                    schema: None,
                    column_type: ColumnType::BigInt,
                },
                "f".to_string(),
            ),
        ],
        TableExpr {
            name: "sxt_tab".to_string(),
        },
        Box::new(NotExpr::new(Box::new(AndExpr::new(
            Box::new(OrExpr::new(
                Box::new(EqualsExpr::new(
                    ColumnRef {
                        column_name: "f".to_string(),
                        table_name: "sxt_tab".to_string(),
                        schema: None,
                        column_type: ColumnType::BigInt,
                    },
                    Scalar::from(45_u64),
                )),
                Box::new(EqualsExpr::new(
                    ColumnRef {
                        column_name: "c".to_string(),
                        table_name: "sxt_tab".to_string(),
                        schema: None,
                        column_type: ColumnType::BigInt,
                    },
                    -Scalar::from(2_u64),
                )),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "b".to_string(),
                    table_name: "sxt_tab".to_string(),
                    schema: None,
                    column_type: ColumnType::BigInt,
                },
                Scalar::from(3_u64),
            )),
        )))),
    );

    let ref_columns = provable_ast.get_column_references();

    assert_eq!(
        ref_columns,
        HashSet::from([
            ColumnRef {
                column_name: "a".to_string(),
                table_name: "sxt_tab".to_string(),
                schema: None,
                column_type: ColumnType::BigInt,
            },
            ColumnRef {
                column_name: "f".to_string(),
                table_name: "sxt_tab".to_string(),
                schema: None,
                column_type: ColumnType::BigInt,
            },
            ColumnRef {
                column_name: "c".to_string(),
                table_name: "sxt_tab".to_string(),
                schema: None,
                column_type: ColumnType::BigInt,
            },
            ColumnRef {
                column_name: "b".to_string(),
                table_name: "sxt_tab".to_string(),
                schema: None,
                column_type: ColumnType::BigInt,
            }
        ])
    );
}
