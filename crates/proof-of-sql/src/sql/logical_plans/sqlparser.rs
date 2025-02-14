use super::{unsupported, LogicalPlanError, LogicalPlanResult, SortExpr};
use crate::{base::database::table_ref::TableRef, sql::logical_plans::LogicalPlan};
use sqlparser::ast::{
    Expr, Ident, ObjectName, Offset, OffsetRows, OrderBy, Query, Select, SetExpr, SetOperator,
    SetQuantifier, Statement, TableFactor, TableWithJoins, Value,
};

pub fn sql_statement_to_plan(statement: &Statement) -> LogicalPlanResult<LogicalPlan> {
    match statement {
        // We do not support anything other than Query yet
        Statement::Query(query) => query_to_plan(*query),
        _ => Err(unsupported(
            "We don't support any Statement other than Query",
        )),
    }
}

/// Convert a [`sqlparser::ast::Query`] to a [`LogicalPlan`].
///
/// Note that we only support a `Query` with `body`, `order_by`, `limit` and `offset`.
fn query_to_plan(query: &Query) -> LogicalPlanResult<LogicalPlan> {
    // First exclude what we don't support
    match (
        query.with,
        query.limit_by,
        query.fetch,
        query.locks,
        query.for_clause,
        query.settings,
        query.format_clause,
    ) {
        (None, None, None, None, None, None, None) => {}
        _ => {
            return Err(unsupported(
                "We only support `body`, `order_by`, `limit` and `offset` in Query",
            ))
        }
    }

    // We only support
    // `body`, `order_by`, `limit` and `offset` in the query
    let opt_offset = query.offset.map(|o| offset_to_int(o));
    let opt_limit = query.limit.map(|l| expr_to_int(l));
    let sort_exprs: Vec<SortExpr> = query
        .order_by
        .map(order_by_to_sort_expr)
        .collect::<LogicalPlanResult<_>>()?;
    let core_plan = set_expr_to_plan(&query.body)?;
    Ok(LogicalPlan::Slice(Slice {
        input: Box::new(LogicalPlan::Sort(Sort {
            input: Box::new(core_plan),
            expr: sort_exprs,
        })),
        offset: opt_offset,
        limit: opt_limit,
    }))
}

/// Convert a [`sqlparser::ast::OrderBy`] to a [`LogicalPlan`].
fn set_expr_to_plan(set_expr: &SetExpr) -> LogicalPlanResult<LogicalPlan> {
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
            LogicalPlan::Union(Union {
                left: Box::new(set_expr_to_plan(left)?),
                right: Box::new(set_expr_to_plan(right)?),
            })
        }
        _ => Err(unsupported("Other SetExpr not supported")),
    }
}

/// Convert a [`sqlparser::ast::Select`] to a [`LogicalPlan`].
fn select_to_plan(select: &Select) -> LogicalPlanResult<LogicalPlan> {
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
fn table_with_joins_to_table_ref(table_with_joins: &TableWithJoins) -> LogicalPlanResult<TableRef> {
    if !table_with_joins.joins.is_empty() {
        return Err(unsupported("We don't support JOINs"));
    }
    table_factor_to_table_ref(table_with_joins.relation)
}

/// Convert a [`sqlparser::ast::TableFactor`] to a [`TableRef`].
fn table_factor_to_table_ref(table_factor: &TableFactor) -> LogicalPlanResult<TableRef> {
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

/// Convert a [`sqlparser::ast::OrderBy`] to a vector of [`SortExpr`]s.
///
/// TODO: Support position-based ordering.
fn order_by_to_sort_expr(order_by: &OrderBy) -> LogicalPlanResult<SortExpr> {
    if order_by.interpolate.is_some() {
        return Err(unsupported("We don't support INTERPOLATE"));
    }
    // No ASC/DESC is equivalent to ASC
    let asc = match order_by_expr.asc {
        Some(true) => true,
        Some(false) => false,
        None => true,
    };
    let sort_expr = SortExpr {
        expr: expr_to_ident(&order_by_expr.expr)?,
        asc,
    };
    Ok((expr_to_ident(order_by_expr.expr)?, asc))
}

/// Convert a [`sqlparser::ast::Offset`] to an integer.
fn offset_to_int(offset: &Offset) -> LogicalPlanResult<i64> {
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
fn expr_to_int(expr: &Expr) -> LogicalPlanResult<i64> {
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
fn expr_to_ident(expr: &Expr) -> LogicalPlanResult<Ident> {
    match expr {
        Expr::Identifier(ident) => Ok(ident.clone()),
        _ => Err(unsupported("We only support Idents")),
    }
}
