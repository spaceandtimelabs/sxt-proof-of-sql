use super::{CompositePolynomialBuilder, SumcheckRandomScalars, SumcheckSubpolynomial};
use crate::{
    base::{polynomial::CompositePolynomial, scalar::Scalar},
    proof_primitive::sumcheck::ProverState,
};

pub fn make_sumcheck_prover_state<S: Scalar>(
    subpolynomials: &[SumcheckSubpolynomial<'_, S>],
    num_vars: usize,
    scalars: &SumcheckRandomScalars<S>,
) -> ProverState<S> {
    ProverState::create(&make_sumcheck_polynomial(subpolynomials, num_vars, scalars))
}

/// Given random multipliers, construct an aggregatated sumcheck polynomial from all
/// the individual subpolynomials.
#[tracing::instrument(name = "proof::make_sumcheck_polynomial", level = "debug", skip_all)]
fn make_sumcheck_polynomial<S: Scalar>(
    subpolynomials: &[SumcheckSubpolynomial<'_, S>],
    num_vars: usize,
    scalars: &SumcheckRandomScalars<S>,
) -> CompositePolynomial<S> {
    let mut builder =
        CompositePolynomialBuilder::new(num_vars, &scalars.compute_entrywise_multipliers());
    for (multiplier, subpoly) in scalars
        .subpolynomial_multipliers
        .iter()
        .zip(subpolynomials.iter())
    {
        subpoly.compose(&mut builder, *multiplier);
    }
    builder.make_composite_polynomial()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            polynomial::{compute_evaluation_vector, CompositePolynomial, MultilinearExtension},
            scalar::Curve25519Scalar,
        },
        sql::proof::SumcheckSubpolynomialType,
    };
    use alloc::boxed::Box;
    use num_traits::{One, Zero};

    #[test]
    fn we_can_form_an_aggregated_sumcheck_polynomial() {
        let mle1 = [1, 2, -1];
        let mle2 = [10i64, 20, 100, 30];
        let mle3 = [2000i64, 3000, 5000, 7000];

        let subpolynomials = &[
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::Identity,
                vec![(-Curve25519Scalar::one(), vec![Box::new(&mle1)])],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::Identity,
                vec![(-Curve25519Scalar::from(10u64), vec![Box::new(&mle2)])],
            ),
            SumcheckSubpolynomial::new(
                SumcheckSubpolynomialType::ZeroSum,
                vec![(Curve25519Scalar::from(9876u64), vec![Box::new(&mle3)])],
            ),
        ];

        let multipliers = [
            Curve25519Scalar::from(5u64),
            Curve25519Scalar::from(2u64),
            Curve25519Scalar::from(50u64),
            Curve25519Scalar::from(25u64),
            Curve25519Scalar::from(11u64),
        ];

        let mut evaluation_vector = vec![Zero::zero(); 4];
        compute_evaluation_vector(&mut evaluation_vector, &multipliers[..2]);

        let poly = make_sumcheck_polynomial(
            subpolynomials,
            2,
            &SumcheckRandomScalars::new(&multipliers, 4, 2),
        );
        let mut expected_poly = CompositePolynomial::new(2);
        let fr = (&evaluation_vector).to_sumcheck_term(2);
        expected_poly.add_product(
            [fr.clone(), (&mle1).to_sumcheck_term(2)],
            -Curve25519Scalar::from(1u64) * multipliers[2],
        );
        expected_poly.add_product(
            [fr, (&mle2).to_sumcheck_term(2)],
            -Curve25519Scalar::from(10u64) * multipliers[3],
        );
        expected_poly.add_product(
            [(&mle3).to_sumcheck_term(2)],
            Curve25519Scalar::from(9876u64) * multipliers[4],
        );
        let random_point = [
            Curve25519Scalar::from(123u64),
            Curve25519Scalar::from(101_112_u64),
        ];
        let eval = poly.evaluate(&random_point);
        let expected_eval = expected_poly.evaluate(&random_point);
        assert_eq!(eval, expected_eval);
    }
}
