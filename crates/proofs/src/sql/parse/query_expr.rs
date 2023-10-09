use super::{FilterExprBuilder, QueryContextBuilder, ResultExprBuilder};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            SchemaAccessor,
        },
        proof::ProofError,
    },
    sql::{
        ast::FilterExpr,
        parse::ConversionResult,
        proof::{
            CountBuilder, ProofBuilder, ProofExpr, ProverEvaluate, TransformExpr,
            VerificationBuilder,
        },
        transform::ResultExpr,
    },
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use proofs_sql::{intermediate_ast::SetExpression, Identifier, SelectStatement};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

#[derive(DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct QueryExpr {
    filter: FilterExpr,
    result: ResultExpr,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and result fields in a readable format.
impl fmt::Debug for QueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueryExpr \n[{:#?},\n{:#?}\n]", self.filter, self.result)
    }
}

impl QueryExpr {
    pub fn new(filter: FilterExpr, result: ResultExpr) -> Self {
        Self { filter, result }
    }

    pub fn try_new(
        ast: SelectStatement,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        let context = match *ast.expr {
            SetExpression::Query {
                result_exprs,
                from,
                where_expr,
                group_by,
            } => QueryContextBuilder::new(schema_accessor)
                .visit_table_expr(from, default_schema)
                .visit_group_by_exprs(group_by)?
                .visit_result_exprs(result_exprs)?
                .visit_where_expr(where_expr)?
                .visit_order_by_exprs(ast.order_by)
                .visit_slice_expr(ast.slice)
                .build()?,
        };

        let filter = FilterExprBuilder::new(context.get_column_mapping())
            .add_table_expr(*context.get_table_ref())
            .add_where_expr(context.get_where_expr().clone())
            .add_result_column_set(context.get_result_column_set())
            .build();

        let result_aliased_exprs = context.get_aliased_result_exprs()?;
        let result = ResultExprBuilder::default()
            .add_group_by_exprs(context.get_group_by_exprs(), result_aliased_exprs)
            .add_select_exprs(result_aliased_exprs)
            .add_order_by_exprs(context.get_order_by_exprs()?)
            .add_slice_expr(context.get_slice_expr())
            .build();

        Ok(Self { filter, result })
    }

    /// Immutable access to this query's provable filter expression.
    pub fn filter(&self) -> &FilterExpr {
        &self.filter
    }

    /// Immutable access to this query's post-proof result transform expression.
    pub fn result(&self) -> &ResultExpr {
        &self.result
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

impl ProverEvaluate for QueryExpr {
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        self.filter.prover_evaluate(builder, alloc, accessor)
    }
}

impl TransformExpr for QueryExpr {
    fn transform_results(&self, result: RecordBatch) -> RecordBatch {
        self.result.transform_results(result)
    }
}
