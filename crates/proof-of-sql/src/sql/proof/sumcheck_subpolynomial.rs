use super::CompositePolynomialBuilder;
use crate::base::{polynomial::MultilinearExtension, scalar::Scalar};
use alloc::{boxed::Box, vec::Vec};

/// The type of a sumcheck subpolynomial
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SumcheckSubpolynomialType {
    /// The subpolynomial should be zero at every entry/row
    Identity,
    /// The subpolynomial should sum to zero across every entry/row
    ZeroSum,
}

/// A term in a sumcheck subpolynomial, represented as a product of multilinear
/// extensions and a constant.
pub type SumcheckSubpolynomialTerm<'a, S> = (S, Vec<Box<dyn MultilinearExtension<S> + 'a>>);

/// A polynomial that can be used to check a contraint and can be aggregated
/// into a single sumcheck polynomial.
/// There are two types of subpolynomials:
/// 1. [`Identity`](SumcheckSubpolynomialType::Identity): the subpolynomial should be zero at every entry/row
/// 2. [`ZeroSum`](SumcheckSubpolynomialType::ZeroSum): the subpolynomial should sum to zero across every entry/row
///
/// The subpolynomial is represented as a sum of terms, where each term is a
/// product of multilinear extensions and a constant.
#[derive(Debug)]
pub struct SumcheckSubpolynomial<'a, S: Scalar> {
    terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    subpolynomial_type: SumcheckSubpolynomialType,
}

impl<'a, S: Scalar> SumcheckSubpolynomial<'a, S> {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    ) -> Self {
        Self {
            terms,
            subpolynomial_type,
        }
    }

    /// Combine the subpolynomial into a combined composite polynomial
    pub fn compose(
        &self,
        composite_polynomial: &mut CompositePolynomialBuilder<S>,
        group_multiplier: S,
    ) {
        for (mult, term) in &self.terms {
            match self.subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    composite_polynomial.produce_fr_multiplicand(&(*mult * group_multiplier), term);
                }
                SumcheckSubpolynomialType::ZeroSum => composite_polynomial
                    .produce_zerosum_multiplicand(&(*mult * group_multiplier), term),
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn subpolynomial_type(&self) -> SumcheckSubpolynomialType {
        self.subpolynomial_type
    }

    /// Returns an iterator over the terms of the subpolynomial, where each term's
    /// coefficient is multiplied by the given multiplier.
    ///
    /// # Arguments
    ///
    /// * `multiplier` - The scalar value to multiply each term's coefficient by.
    ///
    /// # Returns
    ///
    /// An iterator that yields tuples containing the subpolynomial type, the
    /// multiplied coefficient, and a slice of multilinear extensions.
    #[allow(dead_code)]
    pub(crate) fn iter_mul_by(
        &self,
        multiplier: S,
    ) -> impl Iterator<
        Item = (
            SumcheckSubpolynomialType,
            S,
            &[Box<dyn MultilinearExtension<S> + 'a>],
        ),
    > {
        self.terms.iter().map(move |(coeff, multiplicands)| {
            (
                self.subpolynomial_type,
                multiplier * *coeff,
                multiplicands.as_slice(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType};
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::boxed::Box;

    #[test]
    fn test_iter_mul_by() {
        let mle1 = vec![TestScalar::from(1), TestScalar::from(2)];
        let mle2 = vec![TestScalar::from(3), TestScalar::from(4)];

        let terms: Vec<SumcheckSubpolynomialTerm<_>> = vec![
            (TestScalar::from(2), vec![Box::new(&mle1)]),
            (TestScalar::from(3), vec![Box::new(&mle2)]),
        ];
        let subpoly = SumcheckSubpolynomial::new(SumcheckSubpolynomialType::Identity, terms);

        let multiplier = TestScalar::from(5);
        let mut iter = subpoly.iter_mul_by(multiplier);

        let (subpoly_type, coeff, _extensions) = iter.next().unwrap();
        assert_eq!(subpoly_type, SumcheckSubpolynomialType::Identity);
        assert_eq!(coeff, TestScalar::from(10));

        let (subpoly_type, coeff, _extensions) = iter.next().unwrap();
        assert_eq!(subpoly_type, SumcheckSubpolynomialType::Identity);
        assert_eq!(coeff, TestScalar::from(15));

        assert!(iter.next().is_none());
    }
}
