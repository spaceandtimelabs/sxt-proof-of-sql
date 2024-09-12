use crate::proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar;

use super::CompositePolynomialBuilder;
use num_traits::One;

#[test]
fn we_combine_single_degree_fr_multiplicands() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1)]);
    builder.produce_fr_multiplicand(&-Curve25519Scalar::one(), &[Box::new(&mle2)]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1);
    assert_eq!(p.flattened_ml_extensions.len(), 2);
    let pt = [Curve25519Scalar::from(9268764u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 - eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_dont_duplicate_repeated_mles() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle1)]);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3);
    assert_eq!(p.flattened_ml_extensions.len(), 4);
    let pt = [Curve25519Scalar::from(9268764u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval1 + eval1 * eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_combine_identity_with_zero_sum_polynomials() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    builder.produce_zerosum_multiplicand(
        &-Curve25519Scalar::one(),
        &[Box::new(&mle3), Box::new(&mle4)],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [Curve25519Scalar::from(9268764u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval3 = Curve25519Scalar::from(mle3[0]) * m0 + Curve25519Scalar::from(mle3[1]) * m1;
    let eval4 = Curve25519Scalar::from(mle4[0]) * m0 + Curve25519Scalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1 * eval2 - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_only_an_empty_fr_multiplicand() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&Curve25519Scalar::from(17), &[]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1); //1 for the fr multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 2); //1 for fr, 1 for the linear term
    let pt = [Curve25519Scalar::from(9268764u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = (m0 + m1) * Curve25519Scalar::from(17);
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_empty_terms_with_other_terms() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    builder.produce_fr_multiplicand(&Curve25519Scalar::from(17), &[]);
    builder.produce_zerosum_multiplicand(
        &-Curve25519Scalar::one(),
        &[Box::new(&mle3), Box::new(&mle4)],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [Curve25519Scalar::from(9268764u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval3 = Curve25519Scalar::from(mle3[0]) * m0 + Curve25519Scalar::from(mle3[1]) * m1;
    let eval4 = Curve25519Scalar::from(mle4[0]) * m0 + Curve25519Scalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval2 + Curve25519Scalar::from(17)) - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}
