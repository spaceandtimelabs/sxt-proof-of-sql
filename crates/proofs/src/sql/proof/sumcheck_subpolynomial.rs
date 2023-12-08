use super::{CompositePolynomialBuilder, MultilinearExtension};
use crate::base::scalar::ArkScalar;

/// The type of a sumcheck subpolynomial
pub enum SumcheckSubpolynomialType {
    /// The subpolynomial should be zero at every entry/row
    Identity,
    /// The subpolynomial should sum to zero across every entry/row
    ZeroSum,
}

/// A term in a sumcheck subpolynomial, represented as a product of multilinear
/// extensions and a constant.
pub type SumcheckSubpolynomialTerm<'a> = (ArkScalar, Vec<Box<dyn MultilinearExtension + 'a>>);

/// A polynomial that can be used to check a contraint and can be aggregated
/// into a single sumcheck polynomial.
/// There are two types of subpolynomials:
/// 1. Identity: the subpolynomial should be zero at every entry/row
/// 2. ZeroSum: the subpolynomial should sum to zero across every entry/row
///
/// The subpolynomial is represented as a sum of terms, where each term is a
/// product of multilinear extensions and a constant.
pub struct SumcheckSubpolynomial<'a> {
    terms: Vec<SumcheckSubpolynomialTerm<'a>>,
    subpolynomial_type: SumcheckSubpolynomialType,
}

impl<'a> SumcheckSubpolynomial<'a> {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<SumcheckSubpolynomialTerm<'a>>,
    ) -> Self {
        Self {
            terms,
            subpolynomial_type,
        }
    }

    /// Combine the subpolynomial into a combined composite polynomial
    pub fn compose(
        &self,
        composite_polynomial: &mut CompositePolynomialBuilder,
        group_multiplier: ArkScalar,
    ) {
        for (mult, term) in self.terms.iter() {
            match self.subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    composite_polynomial.produce_fr_multiplicand(&(*mult * group_multiplier), term)
                }
                SumcheckSubpolynomialType::ZeroSum => composite_polynomial
                    .produce_zerosum_multiplicand(&(*mult * group_multiplier), term),
            }
        }
    }
}
