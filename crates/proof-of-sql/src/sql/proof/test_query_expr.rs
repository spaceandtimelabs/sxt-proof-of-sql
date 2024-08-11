use super::{
    CountBuilder, ProofBuilder, ProofCounts, ProofExpr, ProverEvaluate, ResultBuilder,
    VerificationBuilder,
};
use crate::base::{
    database::{
        ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
        OwnedTable,
    },
    proof::ProofError,
    scalar::Curve25519Scalar,
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::DynPartialEq;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::{fmt, fmt::Debug};

type ResultFn = Box<
    dyn for<'a> Fn(&mut ResultBuilder<'a>, &'a Bump, &'a dyn DataAccessor<Curve25519Scalar>)
        + Send
        + Sync,
>;

type ProveFn = Box<
    dyn for<'a> Fn(
            &mut ProofBuilder<'a, Curve25519Scalar>,
            &'a Bump,
            &'a dyn DataAccessor<Curve25519Scalar>,
        ) + Send
        + Sync,
>;

type VerifyFn = Box<
    dyn Fn(&mut VerificationBuilder<RistrettoPoint>, &dyn CommitmentAccessor<RistrettoPoint>)
        + Send
        + Sync,
>;

/// A query expression that can mock desired behavior for testing
#[derive(Default, DynPartialEq, Serialize, Deserialize)]
pub struct TestQueryExpr {
    pub table_length: usize,
    pub offset_generators: usize,
    pub counts: ProofCounts,
    #[serde(skip)]
    pub result_fn: Option<ResultFn>,
    #[serde(skip)]
    pub prover_fn: Option<ProveFn>,
    #[serde(skip)]
    pub verifier_fn: Option<VerifyFn>,
}

impl ProofExpr<RistrettoPoint> for TestQueryExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_degree(self.counts.sumcheck_max_multiplicands);
        builder.count_result_columns(self.counts.result_columns);
        builder.count_anchored_mles(self.counts.anchored_mles);
        builder.count_intermediate_mles(self.counts.intermediate_mles);
        builder.count_subpolynomials(self.counts.sumcheck_subpolynomials);
        builder.count_post_result_challenges(self.counts.post_result_challenges);
        Ok(())
    }

    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.table_length
    }

    fn get_offset(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.offset_generators
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<RistrettoPoint>,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
        _result: Option<&OwnedTable<Curve25519Scalar>>,
    ) -> Result<(), ProofError> {
        if let Some(f) = &self.verifier_fn {
            f(builder, accessor);
        }
        Ok(())
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

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
}

impl ProverEvaluate<Curve25519Scalar> for TestQueryExpr {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        if let Some(f) = &self.result_fn {
            f(builder, alloc, accessor);
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, Curve25519Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<Curve25519Scalar>,
    ) {
        if let Some(f) = &self.prover_fn {
            f(builder, alloc, accessor);
        }
    }
}

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
