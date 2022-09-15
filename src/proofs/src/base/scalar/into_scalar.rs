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

macro_rules! uint_into_scalar {
    ($tt:ty) => {
        impl IntoScalar for $tt {
            fn into_scalar(self) -> Scalar {
                Scalar::from(self)
            }
        }
    };
}

macro_rules! int_into_scalar {
    ($it:ty, $ut:ty) => {
        impl IntoScalar for $it {
            fn into_scalar(self) -> Scalar {
                if self >= 0 {
                    Scalar::from(self as $ut)
                } else {
                    -Scalar::from((-self) as $ut)
                }
            }
        }
    };
}

uint_into_scalar!(u8);
uint_into_scalar!(u16);
uint_into_scalar!(u32);
uint_into_scalar!(u64);
int_into_scalar!(i8, u8);
int_into_scalar!(i16, u16);
int_into_scalar!(i32, u32);
int_into_scalar!(i64, u64);
