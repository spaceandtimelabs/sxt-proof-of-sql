use super::{unsupported, ConversionError, ConversionResult, QueryExpr};
use crate::base::database::table_ref::TableRef;
use sqlparser::ast::{
    Expr, Ident, ObjectName, Offset, OffsetRows, OrderByExpr, Query, Select, SetExpr, SetOperator,
    SetQuantifier, Statement, TableFactor, TableWithJoins, Value,
};

pub fn sql_statement_to_plan(statement: &Statement) -> ConversionResult<QueryExpr> {
    match statement {
        // We do not support anything other than Query yet
        Statement::Query(query) => query_to_plan(*query),
        _ => Err(unsupported(
            "We don't support any Statement other than Query",
        )),
    }
}

fn query_to_plan(query: &Query) -> ConversionResult<QueryExpr> {
    // First exclude what we don't support
    if query.with.is_some() {
        return Err(unsupported("We don't support WITH clause"));
    }
    if query.limit_by.is_some() {
        return Err(unsupported("We don't support LIMIT BY clause"));
    }
    if query.fetch.is_some() {
        return Err(unsupported("We don't support FETCH clause"));
    }
    if query.locks.is_some() {
        return Err(unsupported("We don't support LOCK clause"));
    }
    if query.for_clause.is_some() {
        return Err(unsupported("We don't support FOR clause"));
    }
    // We only support
    // `body`, `order_by`, `limit` and `offset` in the query
    let opt_offset = query.offset.map(|o| offset_to_int(o));
    let opt_limit = query.limit.map(|l| expr_to_int(l));
    let order_by_pairs: Vec<(Ident, bool)> = query
        .order_by
        .map(order_by_to_pairs)
        .collect::<ConversionResult<_>>()?;
}

/// Convert a [`sqlparser::ast::OrderBy`] to a [`QueryExpr`].
fn set_expr_to_plan(set_expr: &SetExpr) -> ConversionResult<QueryExpr> {
    match set_expr {
        SetExpr::Select(select) => select_to_plan(*select),
        SetExpr::SetOperation {
            op,
            left,
            right,
            set_quantifier,
        } => {
            if !matches!(
                (op, set_quantifier),
                (SetOperator::Union, SetQuantifier::All)
            ) {
                return Err(unsupported("We only support UNION ALL"));
            }
            //TODO: Support UNION ALL
            todo!("Support UNION ALL")
        }
        _ => Err(unsupported("Other SetExpr not supported")),
    }
}

/// Convert a [`sqlparser::ast::Select`] to a [`QueryExpr`].
fn select_to_plan(select: &Select) -> ConversionResult<QueryExpr> {
    // First exclude what we don't support
    if select.distinct.is_some() {
        return Err(unsupported("We don't support DISTINCT"));
    }
    if select.top.is_some() {
        return Err(unsupported("We don't support TOP"));
    }
    if select.into.is_some() {
        return Err(unsupported("We don't support INTO"));
    }
    if !select.lateral_views.is_empty() {
        return Err(unsupported("We don't support LATERAL VIEWs"));
    }
    if !select.cluster_by.is_empty() {
        return Err(unsupported("We don't support CLUSTER BY"));
    }
    if !select.distribute_by.is_empty() {
        return Err(unsupported("We don't support DISTRIBUTE BY"));
    }
    if !select.sort_by.is_empty() {
        return Err(unsupported("We don't support SORT BY"));
    }
    //TODO: Support HAVING
    if select.having.is_some() {
        return Err(unsupported("We don't support HAVING"));
    }
    if !self.named_window.is_empty() {
        return Err(unsupported("We don't support WINDOW AS"));
    }
    if self.qualify.is_some() {
        return Err(unsupported("We don't support QUALIFY"));
    }
    if self.value_table_mode.is_some() {
        return Err(unsupported("We don't support VALUES"));
    }
    // We only support `projection`, `from`, `selection` and `group_by`
}

/// Convert a [`sqlparser::ast::TableWithJoins`] to joins.
///
/// TODO: Support JOINs
fn table_with_joins_to_table_ref(table_with_joins: &TableWithJoins) -> ConversionResult<TableRef> {
    if !table_with_joins.joins.is_empty() {
        return Err(unsupported("We don't support JOINs"));
    }
    table_factor_to_table_ref(table_with_joins.relation)
}

/// Convert a [`sqlparser::ast::TableFactor`] to a [`TableRef`].
fn table_factor_to_table_ref(table_factor: &TableFactor) -> ConversionResult<TableRef> {
    match table_factor {
        TableFactor::Table { name, .. } => {
            //TODO: Support alias and filter out args we don't support
            name.try_into()
        }
        //TODO: Support Nested Join
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
        } => {
            todo!("Support Nested Join")
        }
        _ => Err(unsupported("We only support `TableFactor::Table`")),
    }
}

/// Convert a [`sqlparser::ast::OrderByExpr`] to a vector of pairs of column names and sort order.
///
/// TODO: Support position-based ordering.
fn order_by_expr_to_pair(order_by_expr: &OrderByExpr) -> ConversionResult<(Ident, bool)> {
    // No ASC/DESC is equivalent to ASC
    let asc = match order_by_expr.asc {
        Some(true) => true,
        Some(false) => false,
        None => true,
    };
    Ok((expr_to_ident(order_by_expr.expr)?, asc))
}

/// Convert a [`sqlparser::ast::Offset`] to an integer.
fn offset_to_int(offset: &Offset) -> ConversionResult<i64> {
    if offset.rows != OffsetRows::None {
        return Err(unsupported(
            "We do not support keywords after OFFSET <num_rows>",
        ));
    }
    expr_to_int(&offset.value)
}

/// Convert a [`sqlparser::ast::Expr`] to an integer.
///
/// We only support some SQL expressions being integers.
/// If the expression is not supported, an error is returned.
fn expr_to_int(expr: &Expr) -> ConversionResult<i64> {
    match expr {
        Expr::Value(value) => match value {
            Value::Number(n) => n
                .parse()
                .map_err(|_e| unsupported("Invalid integer format")),
            _ => Err(unsupported("We only support integer values")),
        },
        _ => Err(unsupported("We only support integer values")),
    }
}

/// Convert an [`sqlparser::ast::Expr`] to an [`sqlparser::ast::Ident`].
///
/// We only support some [`sqlparser::ast::Expr`]s being [`sqlparser::ast::Ident`]s.
/// If the expression is not supported, an error is returned.
fn expr_to_ident(expr: &Expr) -> ConversionResult<Ident> {
    match expr {
        Expr::Identifier(ident) => Ok(ident.clone()),
        _ => Err(unsupported("We only support Idents")),
    }
}
