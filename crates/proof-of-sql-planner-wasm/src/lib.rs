//! JavaScript-facing validation wrapper around the Proof of SQL planner.

mod diagnostic;
mod input;
mod validation;

pub use diagnostic::{PlannerDiagnostic, PlannerDiagnosticKind};
pub use input::{PlannerColumnSchema, PlannerInput};
pub use validation::{validate, ValidationResult};
use wasm_bindgen::prelude::*;

/// Validate SQL with the real Proof of SQL planner.
///
/// The input JSON must match [`PlannerInput`]. The returned JSON represents [`ValidationResult`],
/// including structured failures for malformed input, SQL, schema, and planner errors.
#[must_use]
#[wasm_bindgen(js_name = validateQuery)]
pub fn validate_query(input_json: &str) -> String {
    let result = match serde_json::from_str(input_json) {
        Ok(input) => validate(input),
        Err(error) => ValidationResult::failure(
            Vec::new(),
            PlannerDiagnostic::new(error.to_string(), PlannerDiagnosticKind::InvalidInput),
        ),
    };
    serde_json::to_string(&result).unwrap_or_else(|_| {
        r#"{"ok":false,"referencedTables":[],"error":{"message":"validation result serialization failed","code":"UNKNOWN_PLANNER_ERROR"}}"#.to_string()
    })
}

#[cfg(all(test, target_arch = "wasm32"))]
mod javascript_boundary_tests {
    use super::*;
    use std::collections::BTreeMap;
    use wasm_bindgen_test::*;

    fn input(sql: &str, data_type: &str) -> PlannerInput {
        PlannerInput {
            sql: sql.to_string(),
            schemas: BTreeMap::from([(
                "APP.T".to_string(),
                vec![PlannerColumnSchema {
                    name: "ID".to_string(),
                    data_type: data_type.to_string(),
                }],
            )]),
        }
    }

    fn through_wasm_boundary(input: PlannerInput) -> ValidationResult {
        let input_json = serde_json::to_string(&input).expect("input should serialize");
        let result_json = validate_query(&input_json);
        serde_json::from_str(&result_json).expect("result should deserialize")
    }

    fn assert_boundary_parity(input: PlannerInput) -> ValidationResult {
        let expected = validate(input.clone());
        let actual = through_wasm_boundary(input);
        assert_eq!(actual, expected);
        actual
    }

    #[wasm_bindgen_test]
    fn valid_query_crosses_the_javascript_boundary() {
        let result = assert_boundary_parity(input("SELECT ID FROM APP.T", "BIGINT"));
        assert!(result.ok);
        assert_eq!(result.referenced_tables, ["APP.T"]);
    }

    #[wasm_bindgen_test]
    fn type_dependent_failure_crosses_the_javascript_boundary() {
        let result = assert_boundary_parity(input("SELECT DISTINCT ID FROM APP.T", "VARCHAR"));
        assert_eq!(
            result.error.expect("expected diagnostic").kind,
            PlannerDiagnosticKind::UnsupportedGroupByExpressionType {
                data_type: "VARCHAR".to_string(),
            }
        );
    }

    #[wasm_bindgen_test]
    fn returned_diagnostics_are_json_with_structured_fields() {
        let input_json = serde_json::to_string(&input("SELECT DISTINCT ID FROM APP.T", "VARCHAR"))
            .expect("input should serialize");
        let result: serde_json::Value =
            serde_json::from_str(&validate_query(&input_json)).expect("result should be JSON");

        assert_eq!(result["ok"], false);
        assert_eq!(
            result["error"]["code"],
            "UNSUPPORTED_GROUP_BY_EXPRESSION_TYPE"
        );
        assert_eq!(result["error"]["dataType"], "VARCHAR");
        assert!(result["error"]["message"].is_string());
    }

    #[wasm_bindgen_test]
    fn missing_schema_is_a_result_instead_of_a_trap() {
        let result = assert_boundary_parity(PlannerInput {
            sql: "SELECT ID FROM APP.T".to_string(),
            schemas: BTreeMap::new(),
        });
        assert_eq!(
            result.error.expect("expected diagnostic").kind,
            PlannerDiagnosticKind::MissingSchema {
                table: "APP.T".to_string(),
            }
        );
    }

    #[wasm_bindgen_test]
    fn malformed_input_is_a_structured_validation_failure() {
        let result: ValidationResult =
            serde_json::from_str(&validate_query("not JSON")).expect("result should be JSON");

        assert!(!result.ok);
        assert_eq!(
            result.error.expect("expected diagnostic").kind,
            PlannerDiagnosticKind::InvalidInput
        );
    }
}
