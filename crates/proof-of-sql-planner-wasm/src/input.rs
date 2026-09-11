use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A column supplied to the JavaScript-facing planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerColumnSchema {
    /// Column name as stored by SXT Chain.
    pub name: String,
    /// SQL datatype string as returned by the SXT Chain table-schema runtime API.
    pub data_type: String,
}

/// Input required to validate a query with the Proof of SQL planner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerInput {
    /// SQL text to validate.
    pub sql: String,
    /// Fully-qualified table name to ordered column schema.
    pub schemas: BTreeMap<String, Vec<PlannerColumnSchema>>,
}
