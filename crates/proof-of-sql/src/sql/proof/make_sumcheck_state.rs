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
use itertools::Itertools;
use tracing::Level;

#[tracing::instrument(
    name = "query_proof::make_sumcheck_prover_state",
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
    let optimizer = SumcheckTermOptimizer::new(all_terms, 1 << num_vars);
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
                    .collect_vec(),
            )
        })
        .collect_vec();
    let max_multiplicands = list_of_products
        .iter()
        .map(|(_, p)| p.len())
        .max()
        .unwrap_or(0);
    ProverState::new(
        list_of_products,
        builder.flattened_ml_extensions(),
        num_vars,
        max_multiplicands,
    )
}

struct FlattenedMLEBuilder<'a, S: Scalar> {
    multiplicand_count: usize,
    all_ml_extensions: Vec<&'a dyn MultilinearExtension<S>>,
    entrywise_multipliers: Option<Vec<S>>,
    num_vars: usize,
    lookup: IndexMap<(*const c_void, usize), usize>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::boxed::Box;

    #[test]
    fn we_can_make_sumcheck_prover_state() {
        let mle1 = &[1, 2];
        let mle2 = &[3, 4];

        let subpolynomials = vec![
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (TestScalar::from(101), vec![Box::new(mle1)]),
                    (TestScalar::from(102), vec![Box::new(mle2), Box::new(mle1)]),
                ],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::ZeroSum,
                vec![
                    (TestScalar::from(103), vec![Box::new(mle1)]),
                    (TestScalar::from(104), vec![Box::new(mle2), Box::new(mle1)]),
                ],
            ),
        ];

        let scalars = vec![
            TestScalar::from(201),
            TestScalar::from(202),
            TestScalar::from(203),
        ];
        let random_scalars = SumcheckRandomScalars::new(&scalars, 2, 1);

        let prover_state = make_sumcheck_prover_state(&subpolynomials, 1, &random_scalars);

        assert_eq!(
            prover_state.list_of_products,
            vec![
                (TestScalar::from(103 * 203), vec![1]),
                (TestScalar::from(104 * 203), vec![2, 1]),
                (TestScalar::from(101 * 202), vec![1, 0]),
                (TestScalar::from(102 * 202), vec![2, 1, 0])
            ]
        );
        assert_eq!(
            prover_state.flattened_ml_extensions,
            vec![
                vec![TestScalar::from(1 - 201), TestScalar::from(201)],
                vec![TestScalar::from(1), TestScalar::from(2)],
                vec![TestScalar::from(3), TestScalar::from(4)],
            ]
        );
        assert_eq!(prover_state.num_vars, 1);
        assert_eq!(prover_state.max_multiplicands, 3);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn we_can_make_complex_sumcheck_prover_state() {
        let mle1 = &[0; 0];
        let mle2 = &[1];
        let mle3 = &[2, 3];
        let mle4 = &[4, 5, 6, 7, 8];

        let subpolynomials = vec![
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (TestScalar::from(101), vec![]),
                    (TestScalar::from(102), vec![]),
                    (TestScalar::from(103), vec![Box::new(mle1)]),
                    (TestScalar::from(104), vec![Box::new(mle2)]),
                ],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::Identity,
                vec![
                    (TestScalar::from(105), vec![Box::new(mle2), Box::new(mle3)]),
                    (
                        TestScalar::from(106),
                        vec![Box::new(mle1), Box::new(mle2), Box::new(mle4)],
                    ),
                ],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::ZeroSum,
                vec![
                    (TestScalar::from(107), vec![]),
                    (TestScalar::from(108), vec![]),
                    (TestScalar::from(109), vec![Box::new(mle3)]),
                    (TestScalar::from(110), vec![Box::new(mle4)]),
                ],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::ZeroSum,
                vec![
                    (TestScalar::from(111), vec![Box::new(mle1), Box::new(mle2)]),
                    (
                        TestScalar::from(112),
                        vec![Box::new(mle3), Box::new(mle2), Box::new(mle4)],
                    ),
                ],
            ),
        ];

        let scalars = vec![
            TestScalar::from(201),
            TestScalar::from(202),
            TestScalar::from(203),
            TestScalar::from(204),
            TestScalar::from(205),
            TestScalar::from(206),
            TestScalar::from(207),
        ];
        let random_scalars = SumcheckRandomScalars::new(&scalars, 6, 3);

        let prover_state = make_sumcheck_prover_state(&subpolynomials, 3, &random_scalars);

        assert_eq!(
            prover_state.list_of_products,
            vec![
                (TestScalar::from(111 * 207), vec![1, 2]),
                (TestScalar::from(112 * 207), vec![3, 2, 4]),
                (TestScalar::from(105 * 205), vec![2, 3, 0]),
                (TestScalar::from(106 * 205), vec![1, 2, 4, 0]),
                (TestScalar::from(1), vec![5]),
                (TestScalar::from(1), vec![6, 0]),
            ]
        );
        assert_eq!(
            prover_state.flattened_ml_extensions,
            vec![
                vec![
                    (1 - 201) * (1 - 202) * (1 - 203),
                    201 * (1 - 202) * (1 - 203),
                    (1 - 201) * 202 * (1 - 203),
                    201 * 202 * (1 - 203),
                    (1 - 201) * (1 - 202) * 203,
                    201 * (1 - 202) * 203,
                    0,
                    0
                ],
                vec![0, 0, 0, 0, 0, 0, 0, 0],
                vec![1, 0, 0, 0, 0, 0, 0, 0],
                vec![2, 3, 0, 0, 0, 0, 0, 0],
                vec![4, 5, 6, 7, 8, 0, 0, 0],
                vec![
                    107 * 206 + 108 * 206 + 109 * 206 * 2 + 110 * 206 * 4,
                    107 * 206 + 108 * 206 + 109 * 206 * 3 + 110 * 206 * 5,
                    107 * 206 + 108 * 206 + 110 * 206 * 6,
                    107 * 206 + 108 * 206 + 110 * 206 * 7,
                    107 * 206 + 108 * 206 + 110 * 206 * 8,
                    107 * 206 + 108 * 206,
                    107 * 206 + 108 * 206,
                    107 * 206 + 108 * 206
                ],
                vec![
                    101 * 204 + 102 * 204 + 104 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204,
                    101 * 204 + 102 * 204
                ],
            ]
            .into_iter()
            .map(|v| v.into_iter().map(TestScalar::from).collect_vec())
            .collect_vec()
        );
        assert_eq!(prover_state.num_vars, 3);
        assert_eq!(prover_state.max_multiplicands, 4);
    }
}
