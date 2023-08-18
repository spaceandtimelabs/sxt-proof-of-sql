use crate::sql::proof::{
    CountBuilder, ProofBuilder, ProofExpr, TransformExpr, VerificationBuilder,
};
use std::collections::HashSet;

use crate::base::database::{ColumnField, ColumnRef};
use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};
use crate::base::proof::ProofError;

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
