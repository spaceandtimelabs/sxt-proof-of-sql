use crate::base::{polynomial::MultilinearExtension, scalar::Scalar};

/// This function takes a set of columns and fold it into a slice of scalars.
///
/// The result `res` is updated with
/// `res[i] += mul * sum (beta^j * columns[j][i]) for j in 0..columns.len()`
/// where each column is padded with 0s as needed.
///
/// This is similar to adding `mul * fold_vals(beta,...)` on each row.
pub fn fold_columns<S: Scalar>(
    res: &mut [S],
    mul: S,
    beta: S,
    columns: &[impl MultilinearExtension<S>],
) {
    for (m, col) in powers(mul, beta).zip(columns) {
        col.mul_add(res, &m);
    }
}

/// This function takes a set of values and returns a scalar that is the
/// result of folding the values.
///
/// The result is
/// `sum (beta^j * vals[j]) for j in 0..vals.len()`
pub fn fold_vals<S: Scalar>(beta: S, vals: &[S]) -> S {
    let beta_powers = powers(S::one(), beta);
    beta_powers.zip(vals).map(|(pow, &val)| pow * val).sum()
}

/// Returns an iterator for the lazily evaluated sequence `init, init * base, init * base^2, ...`
fn powers<S: Scalar>(init: S, base: S) -> impl Iterator<Item = S> {
    core::iter::successors(Some(init), move |&m| Some(m * base))
}
