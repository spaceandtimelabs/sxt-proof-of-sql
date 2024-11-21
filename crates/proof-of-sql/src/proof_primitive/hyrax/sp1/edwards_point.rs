use crate::base::scalar::Curve25519Scalar;
use curve25519_dalek::EdwardsPoint;

impl core::ops::Mul<EdwardsPoint> for Curve25519Scalar {
    type Output = EdwardsPoint;
    fn mul(self, rhs: EdwardsPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for EdwardsPoint {
    type Output = EdwardsPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}
impl core::ops::Mul<&EdwardsPoint> for Curve25519Scalar {
    type Output = EdwardsPoint;
    fn mul(self, rhs: &EdwardsPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for &EdwardsPoint {
    type Output = EdwardsPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}
