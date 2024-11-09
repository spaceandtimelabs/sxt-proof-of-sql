use super::{CompositePolynomialBuilder, SumcheckRandomScalars, SumcheckSubpolynomial};
use crate::{base::scalar::Scalar, proof_primitive::sumcheck::ProverState};

/// Given random multipliers, construct an aggregatated sumcheck polynomial from all
/// the individual subpolynomials.
#[tracing::instrument(
    name = "query_proof::make_sumcheck_polynomial",
    level = "debug",
    skip_all
)]
pub fn make_sumcheck_prover_state<S: Scalar>(
    subpolynomials: &[SumcheckSubpolynomial<'_, S>],
    num_vars: usize,
    scalars: &SumcheckRandomScalars<S>,
) -> ProverState<S> {
    let mut builder =
        CompositePolynomialBuilder::new(num_vars, &scalars.compute_entrywise_multipliers());
    for (multiplier, subpoly) in scalars
        .subpolynomial_multipliers
        .iter()
        .zip(subpolynomials.iter())
    {
        subpoly.compose(&mut builder, *multiplier);
    }
    ProverState::create(&builder.make_composite_polynomial())
}
