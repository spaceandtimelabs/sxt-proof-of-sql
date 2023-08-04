mod evaluation_vector;
pub use evaluation_vector::compute_evaluation_vector;
#[cfg(test)]
mod evaluation_vector_test;

mod count_builder;
pub use count_builder::CountBuilder;

mod index_utility;
pub use index_utility::are_indexes_valid;
#[cfg(test)]
mod index_utility_test;

mod multilinear_extension;
pub use multilinear_extension::{MultilinearExtension, MultilinearExtensionImpl};

mod proof_builder;
pub use proof_builder::ProofBuilder;
#[cfg(test)]
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

mod provable_result_column;
pub use provable_result_column::{DenseProvableResultColumn, ProvableResultColumn};

mod provable_query_result;
pub use provable_query_result::ProvableQueryResult;
#[cfg(test)]
mod provable_query_result_test;

mod sumcheck_mle_evaluations;
pub use sumcheck_mle_evaluations::SumcheckMleEvaluations;
#[cfg(test)]
mod sumcheck_mle_evaluations_test;

mod sumcheck_random_scalars;
pub use sumcheck_random_scalars::SumcheckRandomScalars;

mod proof_exprs;
pub use proof_exprs::{ProofExpr, TransformExpr};

mod query_proof;
pub use query_proof::QueryProof;
#[cfg(test)]
mod query_proof_test;

mod query_result;
pub use query_result::{QueryError, QueryResult};

mod sumcheck_subpolynomial;
pub use sumcheck_subpolynomial::SumcheckSubpolynomial;

mod verifiable_query_result;
pub use verifiable_query_result::VerifiableQueryResult;
#[cfg(test)]
mod verifiable_query_result_test;

#[cfg(test)]
mod verifiable_query_result_test_utility;
#[cfg(test)]
pub use verifiable_query_result_test_utility::exercise_verification;

#[cfg(test)]
mod test_query_expr;
#[cfg(test)]
pub use test_query_expr::TestQueryExpr;

mod result_element_serialization;
pub use result_element_serialization::{
    decode_multiple_elements, DecodeProvableResultElement, EncodeProvableResultElement,
};

mod lagrange_basis_evaluation;
pub use lagrange_basis_evaluation::{
    compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
};
#[cfg(test)]
mod lagrange_basis_evaluation_test;
