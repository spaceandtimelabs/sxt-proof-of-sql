use crate::base::scalar::ArkScalar;

use super::{CompositePolynomialBuilder, MultilinearExtension};

pub enum SumcheckSubpolynomialType {
    Identity,
    ZeroSum,
}

/// A polynomial that sums to zero across binary values and can be aggregated
/// into a single sumcheck polynomial
pub struct SumcheckSubpolynomial<'a> {
    terms: Vec<(ArkScalar, Vec<Box<dyn MultilinearExtension + 'a>>)>,
    subpolynomial_type: SumcheckSubpolynomialType,
}

impl<'a> SumcheckSubpolynomial<'a> {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<(ArkScalar, Vec<Box<dyn MultilinearExtension + 'a>>)>,
    ) -> Self {
        Self {
            terms,
            subpolynomial_type,
        }
    }

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
