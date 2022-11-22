use curve25519_dalek::scalar::Scalar;

/// Given a point of evaluation, computes the vector that allows us
/// to evaluate a multilinear extension as an inner product.
#[tracing::instrument(
    name = "proofs.sql.proof.evaluation_vector.compute_evaluation_vector",
    level = "info",
    skip_all
)]
pub fn compute_evaluation_vector(point: &[Scalar]) -> Vec<Scalar> {
    let m = point.len();
    assert!(m > 0);
    let n = 1 << m;
    let mut res = vec![Scalar::one(); n];
    compute_evaluation_vector_impl(&mut res, point);
    res
}

fn compute_evaluation_vector_impl(v: &mut [Scalar], point: &[Scalar]) {
    let m = point.len();
    if m == 1 {
        assert_eq!(v.len(), 2);
        v[0] = Scalar::one() - point[0];
        v[1] = point[0];
        return;
    }
    let n_half = 1 << (m - 1);
    let (left, right) = v.split_at_mut(n_half);
    compute_evaluation_vector_impl(left, &point[0..m - 1]);
    let p = point[m - 1];
    let pm1 = Scalar::one() - p;
    for i in 0..n_half {
        right[i] = left[i] * p;
        left[i] *= pm1;
    }
}
