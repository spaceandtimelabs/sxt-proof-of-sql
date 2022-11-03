use super::{ProofBuilder, ProofCounts, QueryExpr, VerificationBuilder};

use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};

use bumpalo::Bump;
use std::fmt;
use std::fmt::Debug;

type ProveFn = Box<dyn for<'a> Fn(&mut ProofBuilder<'a>, &'a Bump, &'a dyn DataAccessor)>;
type VerifyFn = Box<dyn Fn(&mut VerificationBuilder, &dyn CommitmentAccessor)>;

/// A query expression that can mock desired behavior for testing
#[derive(Default)]
pub struct TestQueryExpr {
    pub counts: ProofCounts,
    pub prove_fn: Option<ProveFn>,
    pub verify_fn: Option<VerifyFn>,
}

impl QueryExpr for TestQueryExpr {
    fn count(&self, counts: &mut ProofCounts, _accessor: &dyn MetadataAccessor) {
        *counts = self.counts;
    }

    fn prove<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        if let Some(f) = &self.prove_fn {
            f(builder, alloc, accessor);
        }
    }

    fn verify(&self, builder: &mut VerificationBuilder, accessor: &dyn CommitmentAccessor) {
        if let Some(f) = &self.verify_fn {
            f(builder, accessor);
        }
    }
}

impl Debug for TestQueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestQueryExpr").finish()
    }
}
