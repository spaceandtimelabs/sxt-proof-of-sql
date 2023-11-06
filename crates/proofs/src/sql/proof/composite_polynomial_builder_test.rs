use super::{CompositePolynomialBuilder, MultilinearExtensionImpl};
use crate::base::scalar::ArkScalar;
use num_traits::One;

#[test]
fn we_combine_single_degree_fr_multiplicands() {
    let fr = [ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &One::one(),
        &[Box::new(MultilinearExtensionImpl::new(&mle1))],
    );
    builder.produce_fr_multiplicand(
        &-ArkScalar::one(),
        &[Box::new(MultilinearExtensionImpl::new(&mle2))],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1);
    assert_eq!(p.flattened_ml_extensions.len(), 2);
    let pt = [ArkScalar::from(9268764u64)];
    let m0 = ArkScalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = ArkScalar::from(mle1[0]) * m0 + ArkScalar::from(mle1[1]) * m1;
    let eval2 = ArkScalar::from(mle2[0]) * m0 + ArkScalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 - eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_dont_duplicate_repeated_mles() {
    let fr = [ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &One::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle1)),
        ],
    );
    builder.produce_fr_multiplicand(
        &One::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle2)),
        ],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3);
    assert_eq!(p.flattened_ml_extensions.len(), 4);
    let pt = [ArkScalar::from(9268764u64)];
    let m0 = ArkScalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = ArkScalar::from(mle1[0]) * m0 + ArkScalar::from(mle1[1]) * m1;
    let eval2 = ArkScalar::from(mle2[0]) * m0 + ArkScalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval1 + eval1 * eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_combine_identity_with_zero_sum_polynomials() {
    let fr = [ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &One::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle2)),
        ],
    );
    builder.produce_zerosum_multiplicand(
        &-ArkScalar::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle3)),
            Box::new(MultilinearExtensionImpl::new(&mle4)),
        ],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [ArkScalar::from(9268764u64)];
    let m0 = ArkScalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = ArkScalar::from(mle1[0]) * m0 + ArkScalar::from(mle1[1]) * m1;
    let eval2 = ArkScalar::from(mle2[0]) * m0 + ArkScalar::from(mle2[1]) * m1;
    let eval3 = ArkScalar::from(mle3[0]) * m0 + ArkScalar::from(mle3[1]) * m1;
    let eval4 = ArkScalar::from(mle4[0]) * m0 + ArkScalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1 * eval2 - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_only_an_empty_fr_multiplicand() {
    let fr = [ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&ArkScalar::from(17), &[]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1); //1 for the fr multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 2); //1 for fr, 1 for the linear term
    let pt = [ArkScalar::from(9268764u64)];
    let m0 = ArkScalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = (m0 + m1) * ArkScalar::from(17);
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_empty_terms_with_other_terms() {
    let fr = [ArkScalar::from(1u64), ArkScalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(
        &One::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle1)),
            Box::new(MultilinearExtensionImpl::new(&mle2)),
        ],
    );
    builder.produce_fr_multiplicand(&ArkScalar::from(17), &[]);
    builder.produce_zerosum_multiplicand(
        &-ArkScalar::one(),
        &[
            Box::new(MultilinearExtensionImpl::new(&mle3)),
            Box::new(MultilinearExtensionImpl::new(&mle4)),
        ],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [ArkScalar::from(9268764u64)];
    let m0 = ArkScalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = ArkScalar::from(mle1[0]) * m0 + ArkScalar::from(mle1[1]) * m1;
    let eval2 = ArkScalar::from(mle2[0]) * m0 + ArkScalar::from(mle2[1]) * m1;
    let eval3 = ArkScalar::from(mle3[0]) * m0 + ArkScalar::from(mle3[1]) * m1;
    let eval4 = ArkScalar::from(mle4[0]) * m0 + ArkScalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval2 + ArkScalar::from(17)) - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}
