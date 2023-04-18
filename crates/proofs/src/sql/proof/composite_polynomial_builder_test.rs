use super::{CompositePolynomialBuilder, MultilinearExtensionImpl};

use crate::base::scalar::ToScalar;
use curve25519_dalek::scalar::Scalar;

#[test]
fn we_combine_single_degree_fr_multiplicands() {
    let fr = [Scalar::from(1u64), Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &Scalar::one(),
        &[Box::new(MultilinearExtensionImpl::new(&mle1))],
    );
    builder.produce_fr_multiplicand(
        &-Scalar::one(),
        &[Box::new(MultilinearExtensionImpl::new(&mle2))],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1);
    assert_eq!(p.flattened_ml_extensions.len(), 2);
    let pt = [Scalar::from(9268764u64)];
    let m0 = Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = mle1[0].to_scalar() * m0 + mle1[1].to_scalar() * m1;
    let eval2 = mle2[0].to_scalar() * m0 + mle2[1].to_scalar() * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 - eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_dont_duplicate_repeated_mles() {
    let fr = [Scalar::from(1u64), Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &Scalar::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle1)),
        ],
    );
    builder.produce_fr_multiplicand(
        &Scalar::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle2)),
        ],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3);
    assert_eq!(p.flattened_ml_extensions.len(), 4);
    let pt = [Scalar::from(9268764u64)];
    let m0 = Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = mle1[0].to_scalar() * m0 + mle1[1].to_scalar() * m1;
    let eval2 = mle2[0].to_scalar() * m0 + mle2[1].to_scalar() * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval1 + eval1 * eval2);
    assert_eq!(p.evaluate(&pt), expected);
}
