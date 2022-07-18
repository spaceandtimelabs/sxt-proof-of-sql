use curve25519_dalek::scalar::Scalar;

/// Provides conversion to [Scalar].
///
/// This conversion is especially important for proofs.
/// Any data type we want to support will need to be able to convert to Scalar.
/// So, this trait may be used as a bound for supported data types.
///
/// We could just use rust's [From] and [Into] traits.
/// However, some types we want to support are foreign, and since Scalar itself is foreign, we
/// won't be able to provide these conversions.
///
/// One solution would be to create a new-type around every foreign type we want to support.
/// The other is to provide a new conversion trait entirely.
///
/// The latter was chosen for two reasons:
/// 1. We can still create new-types if we want to, but we don't have to in simple cases.
/// 2. There may be already-existing conversions for Scalar on types we *don't* want to support.
/// A new trait allows us to be explicit about the types we want to support.
pub trait IntoScalar {
    fn into_scalar(self) -> Scalar;
}

impl IntoScalar for Scalar {
    fn into_scalar(self) -> Scalar {
        self
    }
}

impl IntoScalar for bool {
    fn into_scalar(self) -> Scalar {
        if self {
            Scalar::one()
        } else {
            Scalar::zero()
        }
    }
}

impl IntoScalar for u32 {
    fn into_scalar(self) -> Scalar {
        Scalar::from(self)
    }
}

impl IntoScalar for i64 {
    fn into_scalar(self) -> Scalar {
        if self >= 0 {
            Scalar::from(self as u64)
        } else {
            -Scalar::from(-self as u64)
        }
    }
}
impl IntoScalar for i32 {
    fn into_scalar(self) -> Scalar {
        if self > 0 {
            Scalar::from(self as u32)
        } else {
            -Scalar::from(-self as u32)
        }
    }
}
