use curve25519_dalek::scalar::Scalar;

fn compute_evaluation_vector_impl(left: &mut [Scalar], right: &mut [Scalar], p: &Scalar) {
    let k = std::cmp::min(left.len(), right.len());
    let pm1 = Scalar::one() - p;
    for (li, ri) in left.iter_mut().zip(right.iter_mut()) {
        *ri = *li * p;
        *li *= pm1;
    }
    for li in &mut left[k..] {
        *li *= pm1;
    }
}

/// Given a point of evaluation, computes the vector that allows us
/// to evaluate a multilinear extension as an inner product.
#[tracing::instrument(
    name = "proofs.sql.proof.evaluation_vector.compute_evaluation_vector",
    level = "info",
    skip_all
)]
pub fn compute_evaluation_vector(v: &mut [Scalar], point: &[Scalar]) {
    let m = point.len();
    assert!(m > 0);
    assert!(v.len() <= (1 << m));
    assert!(v.len() > (1 << (m - 1)) || v.len() == 1);
    v[0] = Scalar::one() - point[0];
    if v.len() == 1 {
        return;
    }
    v[1] = point[0];
    for (level, p) in point[1..].iter().enumerate() {
        let (left, right) = v.split_at_mut(1 << (level + 1));
        compute_evaluation_vector_impl(left, right, p);
    }
}
