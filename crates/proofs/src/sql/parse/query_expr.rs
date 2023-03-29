use crate::sql::proof::{ProofBuilder, ProofCounts, ProofExpr, TransformExpr, VerificationBuilder};
use std::collections::HashSet;

use crate::base::database::{ColumnField, ColumnRef};
use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};

use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;

#[derive(Debug, DynPartialEq, PartialEq)]
pub struct QueryExpr {
    filter: Box<dyn ProofExpr>,
    result: Box<dyn TransformExpr>,
}

impl QueryExpr {
    pub fn new(filter: Box<dyn ProofExpr>, result: Box<dyn TransformExpr>) -> Self {
        Self { filter, result }
    }
}

impl ProofExpr for QueryExpr {
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor) {
        self.filter.count(counts, accessor)
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) {
        self.filter
            .prover_evaluate(builder, alloc, counts, accessor)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) {
        self.filter.verifier_evaluate(builder, counts, accessor)
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
