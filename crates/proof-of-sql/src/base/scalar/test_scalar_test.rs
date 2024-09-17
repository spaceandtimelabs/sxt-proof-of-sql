use num_traits::Inv;

use crate::base::scalar::{test_scalar::TestScalar, Scalar};

#[test]
fn we_can_get_test_scalar_constants_from_z_p(){
    assert_eq!(TestScalar::from(0), TestScalar::ZERO);
    assert_eq!(TestScalar::from(1), TestScalar::ONE);
    assert_eq!(TestScalar::from(2), TestScalar::TWO);
    // -1/2 == least upper bound
    assert_eq!(-TestScalar::TWO.inv().unwrap(), TestScalar::MAX_SIGNED);
}