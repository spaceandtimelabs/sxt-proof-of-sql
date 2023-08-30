use crate::sql::proof::{
    CountBuilder, ProofBuilder, ProofExpr, TransformExpr, VerificationBuilder,
};
use std::collections::HashSet;

use super::{FilterExprBuilder, QueryContextBuilder, ResultExprBuilder};
use crate::base::database::{ColumnField, ColumnRef, SchemaAccessor};
use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};
use crate::base::proof::ProofError;
use crate::sql::parse::ConversionResult;

use proofs_sql::intermediate_ast::SetExpression;
use proofs_sql::{Identifier, SelectStatement};

use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use std::fmt;

#[derive(DynPartialEq, PartialEq)]
pub struct QueryExpr {
    filter: Box<dyn ProofExpr>,
    result: Box<dyn TransformExpr>,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and result fields in a readable format.
impl fmt::Debug for QueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueryExpr \n[{:#?},\n{:#?}\n]", self.filter, self.result)
    }
}

impl QueryExpr {
    pub fn new(filter: Box<dyn ProofExpr>, result: Box<dyn TransformExpr>) -> Self {
        Self { filter, result }
    }

    pub fn try_new(
        ast: SelectStatement,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        let context = match *ast.expr {
            SetExpression::Query {
                result_columns,
                from,
                where_expr,
                group_by,
            } => QueryContextBuilder::new(schema_accessor)
                .visit_table_expression(from, default_schema)
                .visit_result_columns(result_columns)?
                .visit_where_expr(where_expr)?
                .visit_group_by(group_by)?
                .visit_order_by(ast.order_by)
                .visit_slice(ast.slice)
                .build()?,
        };

        let filter = FilterExprBuilder::new(context.get_column_mapping())
            .set_table(*context.current_table())
            .set_where_clause(context.get_where_expr().clone())
            .add_referenced_result_columns(context.get_referenced_columns())
            .build();

        let result = ResultExprBuilder::default()
            .add_group_by(context.get_group_by()?, context.get_agg_result_exprs()?)
            .add_select(context.get_result_schema()?)
            .add_order_by(context.get_order_by()?)
            .add_slice(context.get_slice())
            .build();

        Ok(Self {
            filter: Box::new(filter),
            result: Box::new(result),
        })
    }
}

impl ProofExpr for QueryExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.filter.count(builder, accessor)
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        self.filter.get_length(accessor)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        self.filter.get_offset(accessor)
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        self.filter.prover_evaluate(builder, alloc, accessor)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<(), ProofError> {
        self.filter.verifier_evaluate(builder, accessor)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.filter.get_column_result_fields()
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        self.filter.get_column_references()
    }
}

impl TransformExpr for QueryExpr {
    fn transform_results(&self, result: RecordBatch) -> RecordBatch {
        self.result.transform_results(result)
    }
}
