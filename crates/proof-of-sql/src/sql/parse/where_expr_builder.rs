use super::{ConversionError, DynProofExprBuilder};
use crate::{
    base::{
        database::{ColumnRef, ColumnType},
        map::IndexMap,
    },
    sql::proof_exprs::{DynProofExpr, ProofExpr},
};
use alloc::boxed::Box;
use proof_of_sql_parser::intermediate_ast::Expression;
use sqlparser::ast::Ident;

/// Builder that enables building a `proof_of_sql::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
/// intended to be used as the where clause in a filter expression or group by expression.
pub struct WhereExprBuilder<'a> {
    builder: DynProofExprBuilder<'a>,
}
impl<'a> WhereExprBuilder<'a> {
    /// Creates a new `WhereExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Ident, ColumnRef>) -> Self {
        Self {
            builder: DynProofExprBuilder::new(column_mapping),
        }
    }
    /// Builds a `proof_of_sql::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
    /// intended to be used as the where clause in a filter expression or group by expression.
    pub fn build(
        self,
        where_expr: Option<Box<Expression>>,
    ) -> Result<Option<DynProofExpr>, ConversionError> {
        where_expr
            .map(|where_expr| {
                let converted_expr = (*where_expr).into();
                let expr_plan = self.builder.build(&converted_expr)?;
                // Ensure that the expression is a boolean expression
                match expr_plan.data_type() {
                    ColumnType::Boolean => Ok(expr_plan),
                    _ => Err(ConversionError::NonbooleanWhereClause {
                        datatype: expr_plan.data_type(),
                    }),
                }
            })
            .transpose()
    }
}
