use super::{ProofExpr, ProvableQueryResult, QueryData, QueryProof, QueryResult};
use crate::base::{
    database::{
        ColumnField, ColumnType, CommitmentAccessor, DataAccessor, OwnedColumn, OwnedTable,
    },
    proof::ProofError,
    scalar::ArkScalar,
};
use curve25519_dalek::ristretto::RistrettoPoint;
use serde::{Deserialize, Serialize};

/// The result of an sql query along with a proof that the query is valid. The
/// result and proof can be verified using commitments to database columns.
///
/// Note: the query result is stored in an intermediate form rather than the final form
/// the end-user sees. The final form is obtained after verification. Using an
/// intermediate form allows us to handle overflow and certain cases where the final
/// result might use floating point numbers (e.g. `SELECT STDDEV(A) FROM T WHERE B = 0`).
///
/// Below we demonstrate typical usage of VerifiableQueryResult with pseudo-code.
///
/// Here we assume that a verifier only has access to the commitments of database columns. To
/// process a query, the verifier forwards the query to an untrusted
/// prover. The prover has full access to the database and constructs a VerifiableQueryResult that
/// it sends back to the verifier. The verifier checks that the result is valid using its
/// commitments, and constructs the finalized form of the query result.
///
/// ```ignore
/// prover_process_query(database_accessor) {
///       query <- receive_query_from_verifier()
///
///       verifiable_result <- VerifiableQueryResult::new(query, database_accessor)
///             // When we construct VerifiableQueryResult from a query expression, we compute
///             // both the result of the query in intermediate form and the proof of the result
///             // at the same time.
///
///       send_to_verifier(verifiable_result)
/// }
///
/// verifier_process_query(query, commitment_accessor) {
///    verifiable_result <- send_query_to_prover(query)
///
///    verify_result <- verifiable_result.verify(query, commitment_accessor)
///    if verify_result.is_error() {
///         // The prover did something wrong. Perhaps the prover tried to tamper with the query
///         // result or maybe its version of the database was out-of-sync with the verifier's
///         // version.
///         do_verification_error()
///    }
///
///    query_result <- verify_result.query_result()
///    if query_result.is_error() {
///         // The prover processed the query correctly, but the query resulted in an error.
///         // For example, perhaps the query added two 64-bit integer columns together that
///         // resulted in an overflow.
///         do_query_error()
///    }
///
///    do_query_success(query_result)
///         // The prover correctly processed a query and the query succeeded. Now, we can
///         // proceed to use the result.
/// }
/// ```
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct VerifiableQueryResult {
    pub provable_result: Option<ProvableQueryResult>,
    pub proof: Option<QueryProof>,
}

impl VerifiableQueryResult {
    /// Form a `VerifiableQueryResult` from a query expression.
    ///
    /// This function both computes the result of a query and constructs a proof of the results
    /// validity.
    pub fn new(
        expr: &(impl ProofExpr + Serialize),
        accessor: &impl DataAccessor<ArkScalar>,
    ) -> VerifiableQueryResult {
        // a query must have at least one result column; if not, it should
        // have been rejected at the parsing stage.

        // handle the empty case
        if expr.is_empty(accessor) {
            return VerifiableQueryResult {
                provable_result: None,
                proof: None,
            };
        }

        let (proof, res) = QueryProof::new(expr, accessor);
        Self {
            provable_result: Some(res),
            proof: Some(proof),
        }
    }

    /// Verify a `VerifiableQueryResult`. Upon success, this function returns the finalized form of
    /// the query result.
    ///
    /// Note: a verified result can still respresent an error (e.g. overflow), but it is a verified
    /// error.
    ///
    /// Note: This does NOT transform the result!
    pub fn verify(
        &self,
        expr: &(impl ProofExpr + Serialize),
        accessor: &impl CommitmentAccessor<RistrettoPoint>,
    ) -> QueryResult {
        // a query must have at least one result column; if not, it should
        // have been rejected at the parsing stage.

        // handle the empty case
        if expr.is_empty(accessor) {
            if self.provable_result.is_some() || self.proof.is_some() {
                return Err(ProofError::VerificationError(
                    "zero sumcheck variables but non-empty result",
                ))?;
            }

            let result_fields = expr.get_column_result_fields();

            return make_empty_query_result(result_fields);
        }

        if self.provable_result.is_none() || self.proof.is_none() {
            return Err(ProofError::VerificationError(
                "non-zero sumcheck variables but empty result",
            ))?;
        }

        self.proof
            .as_ref()
            .unwrap()
            .verify(expr, accessor, self.provable_result.as_ref().unwrap())
    }
}

fn make_empty_query_result(result_fields: Vec<ColumnField>) -> QueryResult {
    let table = OwnedTable::try_new(
        result_fields
            .iter()
            .map(|field| {
                (
                    field.name(),
                    match field.data_type() {
                        ColumnType::BigInt => OwnedColumn::BigInt(vec![]),
                        ColumnType::Int128 => OwnedColumn::Int128(vec![]),
                        ColumnType::VarChar => OwnedColumn::VarChar(vec![]),
                        ColumnType::Scalar => OwnedColumn::Scalar(vec![]),
                    },
                )
            })
            .collect(),
    )?;
    Ok(QueryData {
        table,
        verification_hash: Default::default(),
    })
}
