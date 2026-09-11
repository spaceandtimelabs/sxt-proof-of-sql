use crate::{
    diagnostic::{PlannerDiagnostic, PlannerDiagnosticKind},
    input::{PlannerColumnSchema, PlannerInput},
};
use indexmap::{IndexMap, IndexSet};
use proof_of_sql::base::{
    database::{ColumnType, SchemaAccessorImpl, TableRef},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};
use proof_of_sql_planner::{
    datafusion_config_no_normalization, get_table_refs_from_statement, sql_to_proof_plans,
    statement_with_uppercase_identifiers,
};
use serde::{Deserialize, Serialize};
use sqlparser::{
    ast::{DataType, ExactNumberInfo, Ident, TimezoneInfo},
    dialect::{GenericDialect, PostgreSqlDialect},
    parser::Parser,
    tokenizer::Token,
};

/// Result of planner validation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Whether the real Proof of SQL planner accepted the query.
    pub ok: bool,
    /// Fully-qualified base tables referenced by the query.
    pub referenced_tables: Vec<String>,
    /// Structured failure when `ok` is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<PlannerDiagnostic>,
}

impl ValidationResult {
    fn success(referenced_tables: Vec<String>) -> Self {
        Self {
            ok: true,
            referenced_tables,
            error: None,
        }
    }

    pub(crate) fn failure(referenced_tables: Vec<String>, error: PlannerDiagnostic) -> Self {
        Self {
            ok: false,
            referenced_tables,
            error: Some(error),
        }
    }
}

fn parse_column_type(sql_type: &str) -> Result<ColumnType, String> {
    let mut parser = Parser::new(&PostgreSqlDialect {})
        .try_with_sql(sql_type)
        .map_err(|error| error.to_string())?;
    let data_type = parser
        .parse_data_type()
        .map_err(|error| error.to_string())?;
    let trailing = parser.peek_token();
    if trailing.token != Token::EOF {
        return Err(format!(
            "unexpected trailing token in SXT Chain column type: {}",
            trailing.token
        ));
    }

    match data_type {
        DataType::Boolean => Ok(ColumnType::Boolean),
        DataType::TinyInt(None) => Ok(ColumnType::TinyInt),
        DataType::SmallInt(None) => Ok(ColumnType::SmallInt),
        DataType::Int(None) | DataType::Integer(None) => Ok(ColumnType::Int),
        DataType::BigInt(None) => Ok(ColumnType::BigInt),
        DataType::Varchar(None) => Ok(ColumnType::VarChar),
        DataType::Binary(None) => Ok(ColumnType::VarBinary),
        DataType::Timestamp(None, TimezoneInfo::None) => Ok(ColumnType::TimestampTZ(
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
        )),
        DataType::Decimal(number_info) => {
            let (precision, scale) = match number_info {
                ExactNumberInfo::Precision(precision) => (precision, 0),
                ExactNumberInfo::PrecisionAndScale(precision, scale) => (precision, scale),
                ExactNumberInfo::None => {
                    return Err("DECIMAL requires precision".to_string());
                }
            };
            let precision = u8::try_from(precision)
                .ok()
                .and_then(|value| Precision::new(value).ok())
                .ok_or_else(|| format!("DECIMAL precision {precision} is outside 1..=75"))?;
            let scale = i8::try_from(scale)
                .map_err(|_| format!("DECIMAL scale {scale} is outside the supported range"))?;
            Ok(ColumnType::Decimal75(precision, scale))
        }
        unsupported => Err(format!("unsupported SXT Chain column type: {unsupported}")),
    }
}

fn schema_for_table(
    table: &str,
    columns: &[PlannerColumnSchema],
) -> Result<Vec<(Ident, ColumnType)>, PlannerDiagnostic> {
    columns
        .iter()
        .map(|column| {
            parse_column_type(&column.data_type)
                .map(|column_type| (Ident::new(column.name.to_uppercase()), column_type))
                .map_err(|message| {
                    PlannerDiagnostic::new(
                        message,
                        PlannerDiagnosticKind::UnsupportedSchemaType {
                            table: table.to_string(),
                            column: column.name.clone(),
                            data_type: column.data_type.clone(),
                        },
                    )
                })
        })
        .collect()
}

fn prepare_schemas(
    table_refs: IndexSet<TableRef>,
    input_schemas: std::collections::BTreeMap<String, Vec<PlannerColumnSchema>>,
) -> Result<SchemaAccessorImpl, PlannerDiagnostic> {
    if let Some(table_ref) = table_refs
        .iter()
        .find(|table_ref| table_ref.schema_id().is_none())
    {
        return Err(PlannerDiagnostic::new(
            format!("table reference {table_ref} must be fully qualified as NAMESPACE.TABLE"),
            PlannerDiagnosticKind::InvalidTableReference,
        ));
    }

    let normalized_input_schemas = input_schemas
        .into_iter()
        .map(|(table, columns)| (table.to_uppercase(), columns))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut schemas = IndexMap::default();
    for table_ref in table_refs {
        let table_name = table_ref.to_string();
        let Some(columns) = normalized_input_schemas.get(&table_name) else {
            return Err(PlannerDiagnostic::new(
                format!("schema is required for {table_name}"),
                PlannerDiagnosticKind::MissingSchema { table: table_name },
            ));
        };
        let column_schema = schema_for_table(&table_name, columns)?;
        schemas.insert(table_ref, column_schema);
    }
    Ok(SchemaAccessorImpl::new(schemas))
}

/// Validate a query using the same planner entry point as the prover service.
#[must_use]
pub fn validate(input: PlannerInput) -> ValidationResult {
    if input.sql.trim().is_empty() {
        return ValidationResult::failure(
            Vec::new(),
            PlannerDiagnostic::new("query is empty", PlannerDiagnosticKind::EmptyQuery),
        );
    }

    let statements = match Parser::parse_sql(&GenericDialect {}, &input.sql) {
        Ok(statements) => statements,
        Err(error) => {
            return ValidationResult::failure(
                Vec::new(),
                PlannerDiagnostic::new(error.to_string(), PlannerDiagnosticKind::SqlParseError),
            );
        }
    };
    let [statement] = statements.as_slice() else {
        return ValidationResult::failure(
            Vec::new(),
            PlannerDiagnostic::new(
                format!("expected one statement, found {}", statements.len()),
                PlannerDiagnosticKind::NotOneStatement {
                    count: statements.len(),
                },
            ),
        );
    };
    let statement = statement_with_uppercase_identifiers(statement.clone());
    let table_refs = match get_table_refs_from_statement(&statement) {
        Ok(table_refs) => table_refs,
        Err(error) => {
            return ValidationResult::failure(
                Vec::new(),
                PlannerDiagnostic::new(
                    error.to_string(),
                    PlannerDiagnosticKind::InvalidTableReference,
                ),
            );
        }
    };
    let referenced_tables = table_refs
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let schemas = match prepare_schemas(table_refs, input.schemas) {
        Ok(schemas) => schemas,
        Err(error) => return ValidationResult::failure(referenced_tables, error),
    };

    match sql_to_proof_plans(
        std::slice::from_ref(&statement),
        &schemas,
        &datafusion_config_no_normalization(),
    ) {
        Ok(_) => ValidationResult::success(referenced_tables),
        Err(error) => ValidationResult::failure(
            referenced_tables,
            PlannerDiagnostic::from_planner_error(&error),
        ),
    }
}
