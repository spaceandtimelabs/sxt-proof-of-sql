use super::{ProofBuilder, ProofCounts, ProofExpr, TransformExpr, VerificationBuilder};
use std::collections::HashSet;

use crate::base::database::{
    ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
};

use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use std::fmt;
use std::fmt::Debug;

type ProveFn = Box<
    dyn for<'a> Fn(&mut ProofBuilder<'a>, &'a Bump, &ProofCounts, &'a dyn DataAccessor)
        + Send
        + Sync,
>;

type VerifyFn =
    Box<dyn Fn(&mut VerificationBuilder, &ProofCounts, &dyn CommitmentAccessor) + Send + Sync>;

/// A query expression that can mock desired behavior for testing
#[derive(Default, DynPartialEq)]
pub struct TestQueryExpr {
    pub counts: ProofCounts,
    pub prover_fn: Option<ProveFn>,
    pub verifier_fn: Option<VerifyFn>,
}

impl ProofExpr for TestQueryExpr {
    fn count(&self, counts: &mut ProofCounts, _accessor: &dyn MetadataAccessor) {
        *counts = self.counts;
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) {
        if let Some(f) = &self.prover_fn {
            f(builder, alloc, counts, accessor);
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) {
        if let Some(f) = &self.verifier_fn {
            f(builder, counts, accessor);
        }
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let num_columns = self.counts.result_columns;
        let mut columns = Vec::with_capacity(num_columns);
        for i in 0..num_columns {
            columns.push(ColumnField::new(
                ("a".to_owned() + (i + 1).to_string().as_str())
                    .parse()
                    .unwrap(),
                ColumnType::BigInt,
            ));
        }
        columns
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
}

impl TransformExpr for TestQueryExpr {}

/// Non-implemented equality. This only exists because of the Ast trait bounds.
impl PartialEq for TestQueryExpr {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
    }
}

impl Debug for TestQueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestQueryExpr").finish()
    }
}
