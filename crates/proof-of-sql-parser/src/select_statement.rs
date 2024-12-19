use super::intermediate_ast::{OrderBy, SetExpression, Slice, TableExpression};
use crate::{sql::SelectStatementParser, Identifier, ParseError, ParseResult, ResourceId};
use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};

/// Representation of a select statement.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SelectStatement {
    /// The query expression
    pub expr: Box<SetExpression>,

    /// Sort order applied to the result rows
    pub order_by: Vec<OrderBy>,

    /// Optional slice clause restricting rows returned
    pub slice: Option<Slice>,
}

impl fmt::Debug for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SelectStatement \n[{:#?},\n{:#?},\n{:#?}\n]",
            self.expr, self.order_by, self.slice
        )
    }
}

impl SelectStatement {
    /// Returns the tables referenced in the query as resource ids.
    #[must_use]
    pub fn get_table_references(&self, default_schema: Identifier) -> Vec<ResourceId> {
        let SetExpression::Query { from, .. } = &*self.expr;
        convert_table_expr_to_resource_id_vector(from, default_schema)
    }
}

impl FromStr for SelectStatement {
    type Err = crate::ParseError;

    fn from_str(query: &str) -> ParseResult<Self> {
        SelectStatementParser::new()
            .parse(query)
            .map_err(|e| ParseError::QueryParseError { error: e.to_string() })
    }
}

/// Converts table expressions to resource IDs.
fn convert_table_expr_to_resource_id_vector(
    table_expressions: &[Box<TableExpression>],
    default_schema: Identifier,
) -> Vec<ResourceId> {
    table_expressions
        .iter()
        .map(|table_expression| {
            let TableExpression::Named { table, schema } = table_expression.as_ref();
            let schema = schema.as_ref().map_or_else(
                || default_schema.name(),
                super::identifier::Identifier::as_str,
            );
            ResourceId::try_new(schema, table.as_str()).unwrap()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::SelectStatementParser;

    #[test]
    fn correct_table_references_with_default_schema() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("ETH").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);
        assert_eq!(ref_tables, [ResourceId::try_new("eth", "tab").unwrap()]);
    }

    #[test]
    fn correct_table_references_with_same_default_and_original_schema() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM SCHEMA.TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("SCHEMA").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);
        assert_eq!(ref_tables, [ResourceId::try_new("schema", "tab").unwrap()]);
    }

    #[test]
    fn correct_table_references_with_different_default_and_original_schema() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("SELECT A FROM SCHEMA.TAB WHERE C = 3")
            .unwrap();
        let default_schema = Identifier::try_new("  ETH  ").unwrap();
        let ref_tables = parsed_query_ast.get_table_references(default_schema);
        assert_eq!(ref_tables, [ResourceId::try_new("schema", "tab").unwrap()]);
    }
}
