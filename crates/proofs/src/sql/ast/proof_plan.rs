use super::FilterExpr;
use crate::sql::proof::{ProofExpr, ProverEvaluate};
use serde::{Deserialize, Serialize};

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ProofPlan {
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    Filter(FilterExpr),
}

impl ProofExpr for ProofPlan {
    fn count(
        &self,
        builder: &mut crate::sql::proof::CountBuilder,
        accessor: &dyn crate::base::database::MetadataAccessor,
    ) -> Result<(), crate::base::proof::ProofError> {
        match self {
            ProofPlan::Filter(expr) => expr.count(builder, accessor),
        }
    }

    fn get_length(&self, accessor: &dyn crate::base::database::MetadataAccessor) -> usize {
        match self {
            ProofPlan::Filter(expr) => expr.get_length(accessor),
        }
    }

    fn get_offset(&self, accessor: &dyn crate::base::database::MetadataAccessor) -> usize {
        match self {
            ProofPlan::Filter(expr) => expr.get_offset(accessor),
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut crate::sql::proof::VerificationBuilder<curve25519_dalek::RistrettoPoint>,
        accessor: &dyn crate::base::database::CommitmentAccessor<curve25519_dalek::RistrettoPoint>,
    ) -> Result<(), crate::base::proof::ProofError> {
        match self {
            ProofPlan::Filter(expr) => expr.verifier_evaluate(builder, accessor),
        }
    }

    fn get_column_result_fields(&self) -> Vec<crate::base::database::ColumnField> {
        match self {
            ProofPlan::Filter(expr) => expr.get_column_result_fields(),
        }
    }

    fn get_column_references(&self) -> std::collections::HashSet<crate::base::database::ColumnRef> {
        match self {
            ProofPlan::Filter(expr) => expr.get_column_references(),
        }
    }
}

impl ProverEvaluate for ProofPlan {
    fn result_evaluate<'a>(
        &self,
        builder: &mut crate::sql::proof::ResultBuilder<'a>,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<crate::base::scalar::ArkScalar>,
    ) {
        match self {
            ProofPlan::Filter(expr) => expr.result_evaluate(builder, alloc, accessor),
        }
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut crate::sql::proof::ProofBuilder<'a, crate::base::scalar::ArkScalar>,
        alloc: &'a bumpalo::Bump,
        accessor: &'a dyn crate::base::database::DataAccessor<crate::base::scalar::ArkScalar>,
    ) {
        match self {
            ProofPlan::Filter(expr) => expr.prover_evaluate(builder, alloc, accessor),
        }
    }
}
