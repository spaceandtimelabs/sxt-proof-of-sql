use curve25519_dalek::scalar::Scalar;

/// Given the points a and b with length nu, we can evaluate the lagrange basis of length 2^nu at the two points.
/// This is what [super::compute_evaluation_vector] does.
/// Call the resulting evaluation vectors A and B. This function computes `sum A[i] * B[i] for i in 0..length`. That is:
/// ```text
/// (1-a[0])(1-a[1])...(1-a[nu-1]) * (1-b[0])(1-b[1])...(1-b[nu-1]) +
/// (a[0])(1-a[1])...(1-a[nu-1]) * (b[0])(1-b[1])...(1-b[nu-1]) +
/// (1-a[0])(a[1])...(1-a[nu-1]) * (1-b[0])(b[1])...(1-b[nu-1]) +
/// (a[0])(a[1])...(1-a[nu-1]) * (b[0])(b[1])...(1-b[nu-1]) + ...
/// ```
pub fn compute_truncated_lagrange_basis_inner_product(
    length: usize,
    a: &[Scalar],
    b: &[Scalar],
) -> Scalar {
    compute_truncated_lagrange_basis_inner_product_impl(length, a, b).0
}

// The returned value from this function is (part, full).
// The full value is what the result would be if it were not truncated. (In other words, if length==2^nu.)
// This can be iteratively used to compute the actual result.
fn compute_truncated_lagrange_basis_inner_product_impl(
    part_length: usize,
    a: &[Scalar],
    b: &[Scalar],
) -> (Scalar, Scalar) {
    let nu = a.len();
    assert_eq!(nu, b.len());
    if nu == 0 {
        assert!(part_length <= 1);
        if part_length == 1 {
            (Scalar::one(), Scalar::one())
        } else {
            (Scalar::zero(), Scalar::one())
        }
    } else {
        // We split the imaginary full evaluation vector in half.
        // This is the value that needs to be multiplied by every element in the first half.
        let first_half_term = (Scalar::one() - a[nu - 1]) * (Scalar::one() - b[nu - 1]);
        // This is the value that needs to be multiplied by every element in the second half.
        let second_half_term = a[nu - 1] * b[nu - 1];
        let half_full_length = 1 << (nu - 1);

        // `sub` referrs to the sub-iteration. (In other words, removing the last variable, cutting this into two halves.)
        let sub_part_length = if part_length >= half_full_length {
            part_length - half_full_length
        } else {
            part_length
        };
        let (sub_part, sub_full) = compute_truncated_lagrange_basis_inner_product_impl(
            sub_part_length,
            &a[..nu - 1],
            &b[..nu - 1],
        );

        // This is the primary iteration formula.
        let part = if part_length >= half_full_length {
            sub_full * first_half_term + sub_part * second_half_term
        } else {
            sub_part * first_half_term
        };
        // This is the iteration formula for the non truncated version.
        let full = sub_full * (first_half_term + second_half_term);
        (part, full)
    }
}

/// Given the point `point` (or `a`) with length nu, we can evaluate the lagrange basis of length 2^nu at that point.
/// This is what [super::compute_evaluation_vector] does.
/// Call the resulting evaluation vector A. This function computes `sum A[i] for i in 0..length`. That is:
/// ```text
/// (1-a[0])(1-a[1])...(1-a[nu-1]) +
/// (a[0])(1-a[1])...(1-a[nu-1]) +
/// (1-a[0])(a[1])...(1-a[nu-1]) +
/// (a[0])(a[1])...(1-a[nu-1]) + ...
/// ```
pub fn compute_truncated_lagrange_basis_sum(length: usize, point: &[Scalar]) -> Scalar {
    let nu = point.len();
    if nu == 0 {
        assert!(length <= 1);
        if length == 1 {
            Scalar::one()
        } else {
            Scalar::zero()
        }
    } else {
        // Note: this is essentially the same as the inner production version.
        // The only different is that the full sum is always 1, regardless of any inputs.

        let first_half_term = Scalar::one() - point[nu - 1];
        let second_half_term = point[nu - 1];
        let half_full_length = 1 << (nu - 1);
        let sub_part_length = if length >= half_full_length {
            length - half_full_length
        } else {
            length
        };
        let sub_part = compute_truncated_lagrange_basis_sum(sub_part_length, &point[..nu - 1]);
        if length >= half_full_length {
            first_half_term + sub_part * second_half_term
        } else {
            sub_part * first_half_term
        }
    }
}
