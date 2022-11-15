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
pub trait IntoScalar: Copy {
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
                    -Scalar::from(-(self as i128) as $ut)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_convert_unsigned_integers_into_scalars() {
        assert_eq!(0_u8.into_scalar(), Scalar::from(0_u8));
        assert_eq!(1_u16.into_scalar(), Scalar::from(1_u16));
        assert_eq!(1234_u32.into_scalar(), Scalar::from(1234_u32));
        assert_eq!(u64::MAX.into_scalar(), Scalar::from(u64::MAX));
    }

    #[test]
    fn we_can_convert_signed_positive_integers_into_scalars() {
        assert_eq!(0_i8.into_scalar(), Scalar::from(0_u8));
        assert_eq!(1_i16.into_scalar(), Scalar::from(1_u16));
        assert_eq!(1234_i32.into_scalar(), Scalar::from(1234_u32));
        assert_eq!(i64::MAX.into_scalar(), Scalar::from(i64::MAX as u64));
    }

    #[test]
    fn we_can_convert_signed_negative_integers_into_scalars() {
        assert_eq!((-1_i16).into_scalar(), -Scalar::from(1_i16 as u16));
        assert_eq!((-1234_i32).into_scalar(), -Scalar::from(1234_i32 as u32));
        assert_eq!((-i64::MAX).into_scalar(), -Scalar::from(i64::MAX as u64));
        assert_eq!(i64::MIN.into_scalar(), -Scalar::from(1_u64 << 63));
    }
}
