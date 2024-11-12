use super::{
    sumcheck_term_optimizer::SumcheckTermOptimizer, SumcheckRandomScalars, SumcheckSubpolynomial,
    SumcheckSubpolynomialType,
};
use crate::{
    base::{map::IndexMap, polynomial::MultilinearExtension, scalar::Scalar},
    proof_primitive::sumcheck::ProverState,
};
use alloc::vec::Vec;
use core::ffi::c_void;
use tracing::Level;

struct FlattenedMLEBuilder<'a, S: Scalar> {
    multiplicand_count: usize,
    all_ml_extensions: Vec<&'a dyn MultilinearExtension<S>>,
    entrywise_multipliers: Option<Vec<S>>,
    num_vars: usize,
    lookup: IndexMap<*const c_void, usize>,
}
impl<'a, S: Scalar> FlattenedMLEBuilder<'a, S> {
    fn new(entrywise_multipliers: Option<Vec<S>>, num_vars: usize) -> Self {
        Self {
            multiplicand_count: entrywise_multipliers.is_some().into(),
            all_ml_extensions: Vec::new(),
            entrywise_multipliers,
            num_vars,
            lookup: IndexMap::default(),
        }
    }
    fn position_or_insert(&mut self, multiplicand: &'a dyn MultilinearExtension<S>) -> usize {
        *self.lookup.entry(multiplicand.id()).or_insert_with(|| {
            self.all_ml_extensions.push(multiplicand);
            self.multiplicand_count += 1;
            self.multiplicand_count - 1
        })
    }
    #[tracing::instrument(
        name = "FlattenedMLEBuilder::flattened_ml_extensions",
        level = "debug",
        skip_all
    )]
    fn flattened_ml_extensions(self) -> Vec<Vec<S>> {
        self.entrywise_multipliers
            .into_iter()
            .map(|mle| (&mle).to_sumcheck_term(self.num_vars))
            .chain(
                self.all_ml_extensions
                    .iter()
                    .map(|mle| mle.to_sumcheck_term(self.num_vars)),
            )
            .collect()
    }
}

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
    let needs_entrywise_multipliers = subpolynomials
        .iter()
        .any(|s| matches!(s.subpolynomial_type(), SumcheckSubpolynomialType::Identity));
    let all_terms = scalars
        .subpolynomial_multipliers
        .iter()
        .zip(subpolynomials)
        .flat_map(|(multiplier, terms)| terms.iter_mul_by(*multiplier));

    // Optimization should be very fast. We put this span to double check this. There is almost no copying being done.
    let span = tracing::span!(Level::DEBUG, "optimize sumcheck terms").entered();
    let optimizer = SumcheckTermOptimizer::new(all_terms, scalars.table_length);
    let optimized_terms = optimizer.terms();
    let optimized_term_iter = optimized_terms.into_iter();
    span.exit();

    let mut builder = FlattenedMLEBuilder::new(
        needs_entrywise_multipliers.then(|| scalars.compute_entrywise_multipliers()),
        num_vars,
    );
    let list_of_products = optimized_term_iter
        .map(|(ty, coeff, term)| {
            (
                coeff,
                term.iter()
                    .map(|multiplicand| builder.position_or_insert(multiplicand.as_ref()))
                    .chain(matches!(ty, SumcheckSubpolynomialType::Identity).then_some(0))
                    .collect(),
            )
        })
        .collect();
    ProverState::new(
        list_of_products,
        builder.flattened_ml_extensions(),
        num_vars,
    )
}
