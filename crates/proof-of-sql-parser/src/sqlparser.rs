//! This module exists to adapt the current parser to `sqlparser`.
use crate::{
    intermediate_ast::{
        AliasedResultExpr, BinaryOperator as PoSqlBinaryOperator, Expression, Literal,
        OrderBy as PoSqlOrderBy, OrderByDirection, SelectResultExpr, SetExpression,
        TableExpression, UnaryOperator as PoSqlUnaryOperator,
    },
    Identifier, ResourceId, SelectStatement,
};
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
};
use core::fmt::Display;
use sqlparser::ast::{
    BinaryOperator, DataType, Expr, Function, FunctionArg, FunctionArgExpr, GroupByExpr, Ident,
    ObjectName, Offset, OffsetRows, OrderByExpr, Query, Select, SelectItem, SetExpr, TableFactor,
    TableWithJoins, TimezoneInfo, UnaryOperator, Value, WildcardAdditionalOptions,
};

/// Convert a number into a [`Expr`].
fn number<T>(val: T) -> Expr
where
    T: Display,
{
    Expr::Value(Value::Number(val.to_string(), false))
}

/// Convert an [`Identifier`] into a [`Expr`].
fn id(id: Identifier) -> Expr {
    Expr::Identifier(id.into())
}

impl From<Identifier> for Ident {
    fn from(id: Identifier) -> Self {
        Ident::new(id.as_str())
    }
}

impl From<ResourceId> for ObjectName {
    fn from(id: ResourceId) -> Self {
        ObjectName(vec![id.schema().into(), id.object_name().into()])
    }
}

impl From<TableExpression> for TableFactor {
    fn from(table: TableExpression) -> Self {
        match table {
            TableExpression::Named { table, schema } => {
                let object_name = if let Some(schema) = schema {
                    ObjectName(vec![schema.into(), table.into()])
                } else {
                    ObjectName(vec![table.into()])
                };
                TableFactor::Table {
                    name: object_name,
                    alias: None,
                    args: None,
                    with_hints: vec![],
                    version: None,
                    partitions: vec![],
                }
            }
        }
    }
}

impl From<Literal> for Expr {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::VarChar(s) => Expr::Value(Value::SingleQuotedString(s)),
            Literal::VarBinary(bytes) => {
                // Convert binary data to hex string for SQL representation
                let hex_string =
                    bytes
                        .iter()
                        .fold(String::with_capacity(bytes.len() * 2), |mut acc, byte| {
                            acc.push_str(&format!("{byte:02x}"));
                            acc
                        });
                Expr::Value(Value::HexStringLiteral(hex_string))
            }
            Literal::BigInt(n) => Expr::Value(Value::Number(n.to_string(), false)),
            Literal::Int128(n) => Expr::Value(Value::Number(n.to_string(), false)),
            Literal::Decimal(n) => Expr::Value(Value::Number(n.to_string(), false)),
            Literal::Boolean(b) => Expr::Value(Value::Boolean(b)),
            Literal::Timestamp(timestamp) => {
                // We currently exclusively store timestamps in UTC.
                Expr::TypedString {
                    data_type: DataType::Timestamp(
                        Some(timestamp.timeunit().into()),
                        TimezoneInfo::None,
                    ),
                    value: timestamp.timestamp().to_string(),
                }
            }
            Literal::Null => Expr::Value(Value::Null),
        }
    }
}

impl From<PoSqlBinaryOperator> for BinaryOperator {
    fn from(op: PoSqlBinaryOperator) -> Self {
        match op {
            PoSqlBinaryOperator::And => BinaryOperator::And,
            PoSqlBinaryOperator::Or => BinaryOperator::Or,
            PoSqlBinaryOperator::Equal => BinaryOperator::Eq,
            PoSqlBinaryOperator::LessThan => BinaryOperator::Lt,
            PoSqlBinaryOperator::GreaterThan => BinaryOperator::Gt,
            PoSqlBinaryOperator::Add => BinaryOperator::Plus,
            PoSqlBinaryOperator::Subtract => BinaryOperator::Minus,
            PoSqlBinaryOperator::Multiply => BinaryOperator::Multiply,
            PoSqlBinaryOperator::Division => BinaryOperator::Divide,
        }
    }
}

impl From<PoSqlUnaryOperator> for UnaryOperator {
    fn from(op: PoSqlUnaryOperator) -> Self {
        match op {
            PoSqlUnaryOperator::Not => UnaryOperator::Not,
        }
    }
}

impl From<PoSqlOrderBy> for OrderByExpr {
    fn from(order_by: PoSqlOrderBy) -> Self {
        let asc = match order_by.direction {
            OrderByDirection::Asc => Some(true),
            OrderByDirection::Desc => Some(false),
        };
        OrderByExpr {
            expr: id(order_by.expr),
            asc,
            nulls_first: None,
        }
    }
}

impl From<Expression> for Expr {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Literal(literal) => literal.into(),
            Expression::Column(identifier) => id(identifier),
            Expression::Unary { op, expr } => Expr::UnaryOp {
                op: op.into(),
                expr: Box::new((*expr).into()),
            },
            Expression::Binary { op, left, right } => Expr::BinaryOp {
                left: Box::new((*left).into()),
                op: op.into(),
                right: Box::new((*right).into()),
            },
            Expression::Wildcard => Expr::Wildcard,
            Expression::Aggregation { op, expr } => Expr::Function(Function {
                name: ObjectName(vec![Ident::new(op.to_string())]),
                args: vec![FunctionArg::Unnamed((*expr).into())],
                filter: None,
                null_treatment: None,
                over: None,
                distinct: false,
                special: false,
                order_by: vec![],
            }),
            Expression::IsNull(expr) => Expr::IsNull(Box::new((*expr).into())),
            Expression::IsNotNull(expr) => Expr::IsNotNull(Box::new((*expr).into())),
            Expression::IsTrue(expr) => Expr::IsTrue(Box::new((*expr).into())),
        }
    }
}

// Note that sqlparser singles out `Wildcard` as a separate case, so we have to handle it separately.
impl From<Expression> for FunctionArgExpr {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Wildcard => FunctionArgExpr::Wildcard,
            _ => FunctionArgExpr::Expr(expr.into()),
        }
    }
}

impl From<SelectResultExpr> for SelectItem {
    fn from(select: SelectResultExpr) -> Self {
        match select {
            SelectResultExpr::ALL => SelectItem::Wildcard(WildcardAdditionalOptions {
                opt_exclude: None,
                opt_except: None,
                opt_rename: None,
                opt_replace: None,
            }),
            SelectResultExpr::AliasedResultExpr(AliasedResultExpr { expr, alias }) => {
                SelectItem::ExprWithAlias {
                    expr: (*expr).into(),
                    alias: alias.into(),
                }
            }
        }
    }
}

impl From<SetExpression> for Select {
    fn from(select: SetExpression) -> Self {
        match select {
            SetExpression::Query {
                result_exprs,
                from,
                where_expr,
                group_by,
            } => Select {
                distinct: None,
                top: None,
                projection: result_exprs.into_iter().map(SelectItem::from).collect(),
                into: None,
                from: from
                    .into_iter()
                    .map(|table_expression| TableWithJoins {
                        relation: (*table_expression).into(),
                        joins: vec![],
                    })
                    .collect(),
                lateral_views: vec![],
                selection: where_expr.map(|expr| (*expr).into()),
                group_by: GroupByExpr::Expressions(group_by.into_iter().map(id).collect()),
                cluster_by: vec![],
                distribute_by: vec![],
                sort_by: vec![],
                having: None,
                named_window: vec![],
                qualify: None,
                value_table_mode: None,
            },
        }
    }
}

impl From<SelectStatement> for Query {
    fn from(select: SelectStatement) -> Self {
        Query {
            with: None,
            body: Box::new(SetExpr::Select(Box::new((*select.expr).into()))),
            order_by: select.order_by.into_iter().map(OrderByExpr::from).collect(),
            limit: select.slice.clone().map(|slice| number(slice.number_rows)),
            limit_by: vec![],
            offset: select.slice.map(|slice| Offset {
                value: number(slice.offset_value),
                rows: OffsetRows::None,
            }),
            fetch: None,
            locks: vec![],
            for_clause: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

    // Check that the intermediate AST can be converted to the SQL parser AST which should functionally match
    // the direct conversion from the SQL string.
    // Note that the `PoSQL` parser has some quirks:
    // - If LIMIT is specified, OFFSET must also be specified so we have to append `OFFSET 0`.
    // - Explicit aliases are mandatory for all columns.
    // In this case we will provide an equivalent query
    // for sqlparser that clearly demonstrates the same functionality.
    fn check_posql_intermediate_ast_to_sqlparser_equivalence(posql_sql: &str, sqlparser_sql: &str) {
        let dialect = PostgreSqlDialect {};
        let posql_ast = posql_sql.parse::<SelectStatement>().unwrap();
        let converted_sqlparser_ast = &Statement::Query(Box::new(Query::from(posql_ast)));
        let direct_sqlparser_ast = &Parser::parse_sql(&dialect, sqlparser_sql).unwrap()[0];
        assert_eq!(converted_sqlparser_ast, direct_sqlparser_ast);
    }

    #[test]
    fn we_can_convert_posql_intermediate_ast_to_sqlparser_with_slight_modification() {
        check_posql_intermediate_ast_to_sqlparser_equivalence(
            "select a, b from t limit 10;",
            "select a as a, b as b from t limit 10 offset 0;",
        );
        check_posql_intermediate_ast_to_sqlparser_equivalence(
            "select timestamp '2024-11-07T04:55:12+00:00' as time from t;",
            "select timestamp(0) '2024-11-07 04:55:12 UTC' as time from t;",
        );
        check_posql_intermediate_ast_to_sqlparser_equivalence(
            "select timestamp '2024-11-07T04:55:12.345+03:00' as time from t;",
            "select timestamp(3) '2024-11-07 01:55:12.345 UTC' as time from t;",
        );
    }

    // Check that PoSQL intermediate AST can be converted to SQL parser AST and that the two are equal.
    // Note that this is a stricter test than the previous one so when quirks are present in the PoSQL AST
    // We will have to use the `check_posql_intermediate_ast_to_sqlparser_equivalence` function.
    fn check_posql_intermediate_ast_to_sqlparser_equality(sql: &str) {
        check_posql_intermediate_ast_to_sqlparser_equivalence(sql, sql);
    }

    #[test]
    fn we_can_convert_posql_intermediate_ast_to_sqlparser() {
        check_posql_intermediate_ast_to_sqlparser_equality("SELECT * FROM t");
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select a as a, 4.7 * b as b from namespace.table where c = 2.5;",
        );
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select a as a, b as b from namespace.table where c = 4;",
        );
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select a as a, b as b from namespace.table where c = 4 order by a desc;",
        );
        check_posql_intermediate_ast_to_sqlparser_equality("select 1 as a, 'Meow' as d, b as b from namespace.table where c = 4 order by a desc limit 10 offset 0;");
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select true as cons, a and b or c > 4 as comp from tab where d = 'Space and Time';",
        );
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select cat as cat, true as cons, max(meow) as max_meow from tab where d = 'Space and Time' group by cat;",
        );
        check_posql_intermediate_ast_to_sqlparser_equality(
            "select cat as cat, sum(a) as s, count(*) as rows from tab where d = 'Space and Time' group by cat;",
        );
    }
}
