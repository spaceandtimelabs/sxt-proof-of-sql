//! Native validation and response-contract tests for the planner wrapper.
#![expect(clippy::missing_panics_doc)]

use proof_of_sql_planner_wasm::{
    validate, validate_query, PlannerColumnSchema, PlannerDiagnosticKind, PlannerInput,
    ValidationResult,
};
use std::collections::BTreeMap;

fn columns(values: &[(&str, &str)]) -> Vec<PlannerColumnSchema> {
    values
        .iter()
        .map(|(name, data_type)| PlannerColumnSchema {
            name: (*name).to_string(),
            data_type: (*data_type).to_string(),
        })
        .collect()
}

fn input(sql: &str, schemas: &[(&str, &[(&str, &str)])]) -> PlannerInput {
    PlannerInput {
        sql: sql.to_string(),
        schemas: schemas
            .iter()
            .map(|(table, schema)| ((*table).to_string(), columns(schema)))
            .collect::<BTreeMap<_, _>>(),
    }
}

fn error_kind(result: &ValidationResult) -> &PlannerDiagnosticKind {
    &result.error.as_ref().expect("expected diagnostic").kind
}

#[test]
fn projections_and_identifier_normalization_are_valid() {
    for result in [
        validate(input(
            "SELECT ID FROM APP.T",
            &[("APP.T", &[("ID", "BIGINT")])],
        )),
        validate(input(
            "select id from app.t",
            &[("app.t", &[("id", "bigint")])],
        )),
    ] {
        assert!(result.ok, "{result:?}");
        assert_eq!(result.referenced_tables, ["APP.T"]);
    }
}

#[test]
fn input_and_parse_failures_are_structured() {
    let cases = [
        ("", PlannerDiagnosticKind::EmptyQuery),
        ("SELECT (", PlannerDiagnosticKind::SqlParseError),
        (
            "SELECT 1; SELECT 2",
            PlannerDiagnosticKind::NotOneStatement { count: 2 },
        ),
        (
            "SELECT ID FROM T",
            PlannerDiagnosticKind::InvalidTableReference,
        ),
    ];

    for (sql, expected) in cases {
        let result = validate(input(sql, &[("T", &[("ID", "BIGINT")])]));
        assert_eq!(error_kind(&result), &expected, "{sql}: {result:?}");
    }
}

#[test]
fn schema_failures_retain_table_column_and_type() {
    let missing = validate(input("SELECT ID FROM APP.T", &[]));
    assert_eq!(
        error_kind(&missing),
        &PlannerDiagnosticKind::MissingSchema {
            table: "APP.T".to_string(),
        }
    );
    assert_eq!(missing.referenced_tables, ["APP.T"]);

    let unsupported = validate(input(
        "SELECT ID FROM APP.T",
        &[("APP.T", &[("ID", "VARCHAR(255)")])],
    ));
    assert_eq!(
        error_kind(&unsupported),
        &PlannerDiagnosticKind::UnsupportedSchemaType {
            table: "APP.T".to_string(),
            column: "ID".to_string(),
            data_type: "VARCHAR(255)".to_string(),
        }
    );
}

#[test]
fn malformed_schema_types_are_rejected_completely() {
    for data_type in [
        "BIGINT garbage",
        "DECIMAL",
        "DECIMAL(0, 0)",
        "DECIMAL(76, 0)",
        "DECIMAL(256, 0)",
        "DECIMAL(20, 128)",
        "FLOAT",
    ] {
        let result = validate(input(
            "SELECT ID FROM APP.T",
            &[("APP.T", &[("ID", data_type)])],
        ));
        assert_eq!(
            error_kind(&result),
            &PlannerDiagnosticKind::UnsupportedSchemaType {
                table: "APP.T".to_string(),
                column: "ID".to_string(),
                data_type: data_type.to_string(),
            }
        );
    }
}

#[test]
fn every_supported_schema_type_can_be_prepared() {
    let result = validate(input(
        "SELECT * FROM APP.T",
        &[(
            "APP.T",
            &[
                ("BOOL_COL", "BOOLEAN"),
                ("TINY_COL", "TINYINT"),
                ("SMALL_COL", "SMALLINT"),
                ("INT_COL", "INT"),
                ("INTEGER_COL", "INTEGER"),
                ("BIG_COL", "BIGINT"),
                ("TEXT_COL", "VARCHAR"),
                ("BYTES_COL", "BINARY"),
                ("DECIMAL_ZERO_SCALE_COL", "DECIMAL(20)"),
                ("DECIMAL_COL", "DECIMAL(20, 4)"),
                ("TIME_COL", "TIMESTAMP"),
            ],
        )],
    ));
    assert!(result.ok, "{result:?}");
}

#[test]
fn currently_supported_predicates_are_accepted() {
    for sql in [
        "SELECT ID FROM APP.T WHERE ID BETWEEN 1 AND 10",
        "SELECT ID FROM APP.T WHERE ID IN (1, 2, 3)",
    ] {
        let result = validate(input(sql, &[("APP.T", &[("ID", "BIGINT")])]));
        assert!(result.ok, "{sql}: {result:?}");
    }
}

#[test]
fn grouping_and_distinct_use_authoritative_aggregate_errors() {
    let varchar = validate(input(
        "SELECT DISTINCT ID FROM APP.T",
        &[("APP.T", &[("ID", "VARCHAR")])],
    ));
    assert_eq!(
        error_kind(&varchar),
        &PlannerDiagnosticKind::UnsupportedGroupByExpressionType {
            data_type: "VARCHAR".to_string(),
        }
    );

    for sql in [
        "SELECT ID, VALUE, COUNT(*) FROM APP.T GROUP BY ID, VALUE",
        "SELECT DISTINCT ID, VALUE FROM APP.T",
    ] {
        let result = validate(input(
            sql,
            &[("APP.T", &[("ID", "BIGINT"), ("VALUE", "BIGINT")])],
        ));
        assert_eq!(
            error_kind(&result),
            &PlannerDiagnosticKind::UnsupportedGroupByExpressionCount { count: 2 },
            "{sql}: {result:?}"
        );
    }
}

#[test]
fn numeric_distinct_and_grouping_are_accepted() {
    for sql in [
        "SELECT DISTINCT ID FROM APP.T",
        "SELECT ID, COUNT(*) FROM APP.T GROUP BY ID",
    ] {
        let result = validate(input(sql, &[("APP.T", &[("ID", "BIGINT")])]));
        assert!(result.ok, "{sql}: {result:?}");
    }
}

#[test]
fn join_failures_retain_the_authoritative_reason() {
    let schema = &[
        ("APP.A", &[("ID", "BIGINT")][..]),
        ("APP.B", &[("ID", "BIGINT"), ("OTHER_ID", "BIGINT")][..]),
    ];

    let left = validate(input(
        "SELECT A.ID FROM APP.A A LEFT JOIN APP.B B ON A.ID = B.ID",
        schema,
    ));
    assert_eq!(
        error_kind(&left),
        &PlannerDiagnosticKind::UnsupportedJoinType {
            join_type: "Left".to_string(),
        }
    );

    let predicate = validate(input(
        "SELECT A.ID FROM APP.A A INNER JOIN APP.B B ON A.ID = B.OTHER_ID",
        schema,
    ));
    assert!(matches!(
        error_kind(&predicate),
        PlannerDiagnosticKind::UnsupportedJoinPredicate { left, right }
            if left.contains("A.ID") && right.contains("B.OTHER_ID")
    ));
}

#[test]
fn missing_join_schema_is_reported_before_planning() {
    let result = validate(input(
        "SELECT A.ID FROM APP.A A JOIN APP.B B ON A.ID = B.ID",
        &[("APP.A", &[("ID", "BIGINT")])],
    ));
    assert_eq!(
        error_kind(&result),
        &PlannerDiagnosticKind::MissingSchema {
            table: "APP.B".to_string(),
        }
    );
}

#[test]
fn unsupported_expression_errors_retain_context() {
    let division = validate(input(
        "SELECT ID / 2 FROM APP.T",
        &[("APP.T", &[("ID", "BIGINT")])],
    ));
    assert_eq!(
        error_kind(&division),
        &PlannerDiagnosticKind::UnsupportedBinaryOperator {
            operator: "/".to_string(),
        }
    );

    let average = validate(input(
        "SELECT AVG(ID) FROM APP.T",
        &[("APP.T", &[("ID", "BIGINT")])],
    ));
    assert_eq!(
        error_kind(&average),
        &PlannerDiagnosticKind::UnsupportedAggregateOperation {
            operation: "Avg".to_string(),
        }
    );

    let like = validate(input(
        "SELECT NAME FROM APP.T WHERE NAME LIKE 'A%'",
        &[("APP.T", &[("NAME", "VARCHAR")])],
    ));
    assert!(matches!(
        error_kind(&like),
        PlannerDiagnosticKind::UnsupportedLogicalExpression { expression }
            if expression.contains("Like")
    ));
}

#[test]
fn unsupported_plan_nodes_retain_the_node_kind() {
    let cases = [
        (
            "SELECT ID FROM APP.T ORDER BY ID",
            "sort",
            vec![("APP.T", vec![("ID", "BIGINT")])],
        ),
        (
            "SELECT ROW_NUMBER() OVER () FROM APP.T",
            "window",
            vec![("APP.T", vec![("ID", "BIGINT")])],
        ),
        (
            "SELECT A.ID FROM APP.A A CROSS JOIN APP.B B",
            "cross join",
            vec![
                ("APP.A", vec![("ID", "BIGINT")]),
                ("APP.B", vec![("ID", "BIGINT")]),
            ],
        ),
    ];

    for (sql, expected_node, owned_schemas) in cases {
        let schemas = owned_schemas
            .iter()
            .map(|(table, schema)| (*table, schema.as_slice()))
            .collect::<Vec<_>>();
        let result = validate(input(sql, &schemas));
        assert_eq!(
            error_kind(&result),
            &PlannerDiagnosticKind::UnsupportedLogicalPlan {
                node: expected_node.to_string(),
            },
            "{sql}: {result:?}"
        );
    }
}

#[test]
fn response_contract_uses_javascript_names_and_flat_error_context() {
    let result = validate(input("SELECT ID FROM APP.T", &[]));
    let json = serde_json::to_value(result).expect("result should serialize");

    assert_eq!(json["ok"], false);
    assert_eq!(json["referencedTables"][0], "APP.T");
    assert_eq!(json["error"]["code"], "MISSING_SCHEMA");
    assert_eq!(json["error"]["table"], "APP.T");
    assert!(json["error"]["message"].is_string());
}

#[test]
fn json_string_api_round_trips_the_validation_contract() {
    let input = input(
        "SELECT DISTINCT ID FROM APP.T",
        &[("APP.T", &[("ID", "VARCHAR")])],
    );
    let input_json = serde_json::to_string(&input).expect("input should serialize");
    let result: serde_json::Value =
        serde_json::from_str(&validate_query(&input_json)).expect("result should be JSON");

    assert_eq!(result["ok"], false);
    assert_eq!(
        result["error"]["code"],
        "UNSUPPORTED_GROUP_BY_EXPRESSION_TYPE"
    );
    assert_eq!(result["error"]["dataType"], "VARCHAR");
}

#[test]
fn malformed_json_uses_the_validation_result_contract() {
    let result: ValidationResult =
        serde_json::from_str(&validate_query("not JSON")).expect("result should be JSON");

    assert!(!result.ok);
    assert_eq!(result.referenced_tables, Vec::<String>::new());
    assert_eq!(
        result.error.expect("expected diagnostic").kind,
        PlannerDiagnosticKind::InvalidInput
    );
}
