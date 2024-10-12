use crate::base::{polynomial::CompositePolynomial, scalar::Scalar};
use core::iter;
use itertools::Itertools;

pub struct SumcheckTestCase<S: Scalar> {
    pub polynomial: CompositePolynomial<S>,
    pub num_vars: usize,
    pub max_multiplicands: usize,
    pub sum: S,
}

impl<S: Scalar> SumcheckTestCase<S> {
    fn rand(
        num_vars: usize,
        max_multiplicands: usize,
        products: impl IntoIterator<Item = impl IntoIterator<Item = usize>>,
        rng: &mut (impl ark_std::rand::Rng + ?Sized),
    ) -> Self {
        let length = 1 << num_vars;
        let products_vec: Vec<Vec<usize>> = products
            .into_iter()
            .map(|p| p.into_iter().collect())
            .collect();
        let num_multiplicands = products_vec
            .iter()
            .map(|p| p.iter().max().copied().unwrap_or(0))
            .max()
            .unwrap_or(0)
            + 1;
        let polynomial = CompositePolynomial::<S>::rand(
            num_vars,
            max_multiplicands,
            iter::repeat(length).take(num_multiplicands),
            products_vec,
            rng,
        );
        let sum = polynomial.hypercube_sum(length);
        Self {
            polynomial,
            num_vars,
            max_multiplicands,
            sum,
        }
    }
}

pub fn sumcheck_test_cases<S: Scalar>(
    rng: &mut (impl ark_std::rand::Rng + ?Sized),
) -> impl Iterator<Item = SumcheckTestCase<S>> + '_ {
    (1..=8)
        .cartesian_product(0..=5)
        .flat_map(|(num_vars, max_multiplicands)| {
            [
                Some(vec![]),
                Some(vec![vec![]]),
                (max_multiplicands >= 1).then_some(vec![vec![0]]),
                (max_multiplicands >= 2).then_some(vec![vec![0, 1]]),
                (max_multiplicands >= 3).then_some(vec![
                    vec![0, 1, 2],
                    vec![3, 4],
                    vec![0],
                    vec![],
                ]),
                (max_multiplicands >= 5).then_some(vec![
                    vec![7, 0],
                    vec![2, 4, 8, 5],
                    vec![],
                    vec![3],
                    vec![1, 0, 8, 5, 0],
                    vec![3, 6, 9, 9],
                    vec![7, 8, 3],
                    vec![4, 3, 2],
                    vec![],
                    vec![9, 8, 2],
                ]),
                (max_multiplicands >= 3).then_some(vec![
                    vec![],
                    vec![1, 0],
                    vec![3, 6, 1],
                    vec![],
                    vec![],
                    vec![1, 8],
                    vec![1],
                    vec![8],
                    vec![6, 6],
                    vec![4, 6, 7],
                ]),
            ]
            .into_iter()
            .flatten()
            .map(move |products| (num_vars, max_multiplicands, products))
        })
        .map(|(num_vars, max_multiplicands, products)| {
            SumcheckTestCase::rand(num_vars, max_multiplicands, products, rng)
        })
}
