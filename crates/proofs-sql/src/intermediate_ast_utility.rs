use super::intermediate_ast::{SelectStatement, SetExpression, TableExpression};
use std::ops::Deref;

/// Case-insensitive name and namespace of a table.
#[derive(Debug, Eq, PartialEq)]
pub struct TableRef {
    table_name: String,
    namespace: Option<String>,
}

fn ast_table_exprs_to_vec_table_ref(table_expressions: &[Box<TableExpression>]) -> Vec<TableRef> {
    let mut tables = Vec::new();

    for table_expression in table_expressions.iter() {
        let table_ref: &TableExpression = table_expression.deref();

        match table_ref {
            TableExpression::Named { table, namespace } => {
                let table_name = table.as_str().to_string();
                let namespace = namespace
                    .as_ref()
                    .map(|namespace| namespace.as_str().to_string());

                tables.push(TableRef {
                    table_name,
                    namespace,
                });
            }
        }
    }

    tables
}

/// This function returns the referenced tables in the provided intermediate_ast
///
/// Return:
/// - The vector with all tables referenced by the intermediate ast.
pub fn get_ref_tables_from_ast(intermediate_ast: &SelectStatement) -> Vec<TableRef> {
    let set_expression: &SetExpression = &(intermediate_ast.expr);

    match set_expression {
        SetExpression::Query {
            columns: _,
            from,
            where_expr: _,
        } => ast_table_exprs_to_vec_table_ref(&from[..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::SelectStatementParser;

    #[test]
    fn we_can_get_one_ref_from_a_parsed_query_with_one_table() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("select a from tab where c = 3")
            .unwrap();
        let ref_tables = get_ref_tables_from_ast(&parsed_query_ast);

        // note: the parsed table is always upper case
        assert_eq!(
            ref_tables,
            [TableRef {
                table_name: "TAB".to_string(),
                namespace: None
            }]
        );
    }

    #[test]
    fn we_can_get_one_ref_from_a_parsed_query_with_one_namespaced_table() {
        let parsed_query_ast = SelectStatementParser::new()
            .parse("select a from namespace.tab where c = 3")
            .unwrap();
        let ref_tables = get_ref_tables_from_ast(&parsed_query_ast);

        // note: the parsed table is always upper case
        assert_eq!(
            ref_tables,
            [TableRef {
                table_name: "TAB".to_string(),
                namespace: Some("NAMESPACE".to_string())
            }]
        );
    }
}
