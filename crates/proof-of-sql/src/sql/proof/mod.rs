//! TODO: add docs

mod final_round_builder;
pub(crate) use final_round_builder::FinalRoundBuilder;
#[cfg(all(test, feature = "blitzar"))]
mod final_round_builder_test;

mod composite_polynomial_builder;
pub(crate) use composite_polynomial_builder::CompositePolynomialBuilder;
#[cfg(test)]
mod composite_polynomial_builder_test;

mod verification_builder;
pub(crate) use verification_builder::{StandardVerificationBuilder, VerificationBuilder};
#[cfg(test)]
mod verification_builder_test;

mod provable_result_column;
pub(crate) use provable_result_column::ProvableResultColumn;

mod provable_query_result;
pub use provable_query_result::ProvableQueryResult;

mod sumcheck_mle_evaluations;
pub(crate) use sumcheck_mle_evaluations::SumcheckMleEvaluations;
#[cfg(test)]
mod sumcheck_mle_evaluations_test;

mod sumcheck_random_scalars;
pub(crate) use sumcheck_random_scalars::SumcheckRandomScalars;

mod proof_plan;
pub use proof_plan::ProofPlan;
pub(crate) use proof_plan::{HonestProver, ProverEvaluate, ProverHonestyMarker};

mod query_proof;
use query_proof::QueryProof;
#[cfg(all(test, feature = "blitzar"))]
mod query_proof_test;

mod query_result;
pub use query_result::{QueryData, QueryError, QueryResult};

mod sumcheck_subpolynomial;
pub(crate) use sumcheck_subpolynomial::{
    SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType,
};

mod verifiable_query_result;
pub use verifiable_query_result::VerifiableQueryResult;
#[cfg(all(test, feature = "blitzar"))]
mod verifiable_query_result_test;

#[cfg(all(test, feature = "blitzar"))]
mod verifiable_query_result_test_utility;
#[cfg(all(test, feature = "blitzar"))]
pub(crate) use verifiable_query_result_test_utility::exercise_verification;

mod result_element_serialization;
pub(crate) use result_element_serialization::{
    decode_and_convert, decode_multiple_elements, ProvableResultElement,
};

mod first_round_builder;
pub(crate) use first_round_builder::FirstRoundBuilder;
#[cfg(all(test, feature = "blitzar"))]
mod first_round_builder_test;

#[cfg(all(test, feature = "arrow"))]
mod provable_query_result_test;

mod make_sumcheck_state;

mod sumcheck_term_optimizer;
