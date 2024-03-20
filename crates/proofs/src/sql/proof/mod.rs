mod count_builder;
pub use count_builder::CountBuilder;

mod proof_builder;
pub use proof_builder::ProofBuilder;
#[cfg(all(test, feature = "blitzar"))]
mod proof_builder_test;

mod composite_polynomial_builder;
pub use composite_polynomial_builder::CompositePolynomialBuilder;
#[cfg(test)]
mod composite_polynomial_builder_test;

mod proof_counts;
pub use proof_counts::ProofCounts;

mod verification_builder;
pub use verification_builder::VerificationBuilder;
#[cfg(test)]
mod verification_builder_test;

#[warn(missing_docs)]
mod provable_result_column;
pub use provable_result_column::ProvableResultColumn;

#[warn(missing_docs)]
mod provable_query_result;
pub use provable_query_result::ProvableQueryResult;
#[cfg(test)]
mod provable_query_result_test;

#[warn(missing_docs)]
mod sumcheck_mle_evaluations;
pub use sumcheck_mle_evaluations::SumcheckMleEvaluations;
#[cfg(test)]
mod sumcheck_mle_evaluations_test;

mod sumcheck_random_scalars;
pub use sumcheck_random_scalars::SumcheckRandomScalars;

mod proof_exprs;
pub(crate) use proof_exprs::ProverHonestyMarker;
pub use proof_exprs::{HonestProver, ProofExpr, ProverEvaluate, TransformExpr};

mod query_proof;
#[cfg(test)]
pub use query_proof::make_transcript;
pub use query_proof::QueryProof;
#[cfg(all(test, feature = "blitzar"))]
mod query_proof_test;

#[warn(missing_docs)]
mod query_result;
pub use query_result::{QueryData, QueryError, QueryResult};

#[warn(missing_docs)]
mod sumcheck_subpolynomial;
pub use sumcheck_subpolynomial::{
    SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType,
};

mod verifiable_query_result;
pub use verifiable_query_result::VerifiableQueryResult;
#[cfg(all(test, feature = "blitzar"))]
mod verifiable_query_result_test;

#[cfg(all(test, feature = "blitzar"))]
mod verifiable_query_result_test_utility;
#[cfg(all(test, feature = "blitzar"))]
pub use verifiable_query_result_test_utility::exercise_verification;

#[cfg(test)]
mod test_query_expr;
#[cfg(test)]
pub use test_query_expr::TestQueryExpr;

mod result_element_serialization;
pub use result_element_serialization::{
    decode_and_convert, decode_multiple_elements, ProvableResultElement,
};

#[warn(missing_docs)]
mod indexes;
pub use indexes::Indexes;
#[cfg(test)]
mod indexes_test;

#[warn(missing_docs)]
mod result_builder;
pub use result_builder::ResultBuilder;
