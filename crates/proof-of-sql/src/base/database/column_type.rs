use crate::base::{
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
};
use core::{
    fmt,
    fmt::{Display, Formatter},
    mem::size_of,
};
use serde::{Deserialize, Serialize};

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum ColumnType {
    /// Mapped to bool
    #[serde(alias = "BOOLEAN", alias = "boolean")]
    Boolean,
    /// Mapped to u8
    #[serde(alias = "UINT8", alias = "uint8")]
    Uint8,
    /// Mapped to i8
    #[serde(alias = "TINYINT", alias = "tinyint")]
    TinyInt,
    /// Mapped to i16
    #[serde(alias = "SMALLINT", alias = "smallint")]
    SmallInt,
    /// Mapped to i32
    #[serde(alias = "INT", alias = "int")]
    Int,
    /// Mapped to i64
    #[serde(alias = "BIGINT", alias = "bigint")]
    BigInt,
    /// Mapped to i128
    #[serde(rename = "Decimal", alias = "DECIMAL", alias = "decimal")]
    Int128,
    /// Mapped to String
    #[serde(alias = "VARCHAR", alias = "varchar")]
    VarChar,
    /// Mapped to i256
    #[serde(rename = "Decimal75", alias = "DECIMAL75", alias = "decimal75")]
    Decimal75(Precision, i8),
    /// Mapped to i64
    #[serde(alias = "TIMESTAMP", alias = "timestamp")]
    #[cfg_attr(test, proptest(skip))]
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone),
    /// Mapped to `S`
    #[serde(alias = "SCALAR", alias = "scalar")]
    #[cfg_attr(test, proptest(skip))]
    Scalar,
    /// Mapped to [u8]
    #[serde(alias = "BINARY", alias = "BINARY")]
    VarBinary,
}

impl ColumnType {
    /// Returns true if this column is numeric and false otherwise
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ColumnType::Uint8
                | ColumnType::TinyInt
                | ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
                | ColumnType::Scalar
                | ColumnType::Decimal75(_, _)
        )
    }

    /// Returns true if this column is an integer and false otherwise
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ColumnType::Uint8
                | ColumnType::TinyInt
                | ColumnType::SmallInt
                | ColumnType::Int
                | ColumnType::BigInt
                | ColumnType::Int128
        )
    }

    /// Returns the floor of the sqrt of the negative min integer.
    /// Returns `None` if the type is not a signed integer.
    /// `sqrt_negative_min(NumericalType) = floor(sqrt(-NumericalType::MIN))`
    #[must_use]
    #[cfg_attr(not(test), expect(dead_code))]
    #[expect(clippy::trivially_copy_pass_by_ref)]
    fn sqrt_negative_min(&self) -> Option<u64> {
        match self {
            ColumnType::TinyInt => Some(11),
            ColumnType::SmallInt => Some(181),
            ColumnType::Int => Some(46_340),
            ColumnType::BigInt => Some(3_037_000_499),
            ColumnType::Int128 => Some(13_043_817_825_332_782_212),
            _ => None,
        }
    }

    /// Returns the number of bits in the integer type if it is an integer type. Otherwise, return None.
    fn to_integer_bits(self) -> Option<usize> {
        match self {
            ColumnType::Uint8 | ColumnType::TinyInt => Some(8),
            ColumnType::SmallInt => Some(16),
            ColumnType::Int => Some(32),
            ColumnType::BigInt => Some(64),
            ColumnType::Int128 => Some(128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the signed integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.
    fn from_signed_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnType::TinyInt),
            16 => Some(ColumnType::SmallInt),
            32 => Some(ColumnType::Int),
            64 => Some(ColumnType::BigInt),
            128 => Some(ColumnType::Int128),
            _ => None,
        }
    }

    /// Returns the [`ColumnType`] of the unsigned integer type with the given number of bits if it is a valid integer type.
    ///
    /// Otherwise, return None.
    fn from_unsigned_integer_bits(bits: usize) -> Option<Self> {
        match bits {
            8 => Some(ColumnType::Uint8),
            _ => None,
        }
    }

    /// Returns the larger integer type of two [`ColumnType`]s if they are both integers.
    ///
    /// If either of the columns is not an integer, return None.
    #[must_use]
    pub fn max_integer_type(&self, other: &Self) -> Option<Self> {
        // If either of the columns is not an integer, return None
        if !self.is_integer() || !other.is_integer() {
            return None;
        }
        self.to_integer_bits().and_then(|self_bits| {
            other
                .to_integer_bits()
                .and_then(|other_bits| Self::from_signed_integer_bits(self_bits.max(other_bits)))
        })
    }

    /// Returns the larger integer type of two [`ColumnType`]s if they are both integers.
    ///
    /// If either of the columns is not an integer, return None.
    #[must_use]
    pub fn max_unsigned_integer_type(&self, other: &Self) -> Option<Self> {
        // If either of the columns is not an integer, return None
        if !self.is_integer() || !other.is_integer() {
            return None;
        }
        self.to_integer_bits().and_then(|self_bits| {
            other
                .to_integer_bits()
                .and_then(|other_bits| Self::from_unsigned_integer_bits(self_bits.max(other_bits)))
        })
    }

    /// Returns the precision of a [`ColumnType`] if it is converted to a decimal wrapped in `Some()`. If it can not be converted to a decimal, return None.
    #[must_use]
    pub fn precision_value(&self) -> Option<u8> {
        match self {
            Self::Uint8 | Self::TinyInt => Some(3_u8),
            Self::SmallInt => Some(5_u8),
            Self::Int => Some(10_u8),
            Self::BigInt | Self::TimestampTZ(_, _) => Some(19_u8),
            Self::Int128 => Some(39_u8),
            Self::Decimal75(precision, _) => Some(precision.value()),
            // Scalars are not in database & are only used for typeless comparisons for testing so we return 0
            // so that they do not cause errors when used in comparisons.
            Self::Scalar => Some(0_u8),
            Self::Boolean | Self::VarChar | Self::VarBinary => None,
        }
    }
    /// Returns scale of a [`ColumnType`] if it is convertible to a decimal wrapped in `Some()`. Otherwise return None.
    #[must_use]
    pub fn scale(&self) -> Option<i8> {
        match self {
            Self::Decimal75(_, scale) => Some(*scale),
            Self::TinyInt
            | Self::Uint8
            | Self::SmallInt
            | Self::Int
            | Self::BigInt
            | Self::Int128
            | Self::Scalar => Some(0),
            Self::Boolean | Self::VarBinary | Self::VarChar => None,
            Self::TimestampTZ(tu, _) => match tu {
                PoSQLTimeUnit::Second => Some(0),
                PoSQLTimeUnit::Millisecond => Some(3),
                PoSQLTimeUnit::Microsecond => Some(6),
                PoSQLTimeUnit::Nanosecond => Some(9),
            },
        }
    }

    /// Returns the byte size of the column type.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        match self {
            Self::Boolean => size_of::<bool>(),
            Self::Uint8 => size_of::<u8>(),
            Self::TinyInt => size_of::<i8>(),
            Self::SmallInt => size_of::<i16>(),
            Self::Int => size_of::<i32>(),
            Self::BigInt | Self::TimestampTZ(_, _) => size_of::<i64>(),
            Self::Int128 => size_of::<i128>(),
            Self::Scalar | Self::Decimal75(_, _) | Self::VarBinary | Self::VarChar => {
                size_of::<[u64; 4]>()
            }
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    /// Returns the bit size of the column type.
    #[must_use]
    pub fn bit_size(&self) -> u32 {
        self.byte_size() as u32 * 8
    }

    /// Returns if the column type supports signed values.
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        match self {
            Self::TinyInt
            | Self::SmallInt
            | Self::Int
            | Self::BigInt
            | Self::Int128
            | Self::TimestampTZ(_, _) => true,
            Self::Decimal75(_, _)
            | Self::Scalar
            | Self::VarBinary
            | Self::VarChar
            | Self::Boolean
            | Self::Uint8 => false,
        }
    }

    /// Returns if the column type supports signed values.
    #[must_use]
    pub fn min_scalar<S: Scalar>(&self) -> Option<S> {
        match self {
            ColumnType::TinyInt => Some(S::from(i8::MIN)),
            ColumnType::SmallInt => Some(S::from(i16::MIN)),
            ColumnType::Int => Some(S::from(i32::MIN)),
            ColumnType::BigInt => Some(S::from(i64::MIN)),
            ColumnType::Int128 => Some(S::from(i128::MIN)),
            _ => None,
        }
    }
}

/// Display the column type as a str name (in all caps)
impl Display for ColumnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Boolean => write!(f, "BOOLEAN"),
            ColumnType::Uint8 => write!(f, "UINT8"),
            ColumnType::TinyInt => write!(f, "TINYINT"),
            ColumnType::SmallInt => write!(f, "SMALLINT"),
            ColumnType::Int => write!(f, "INT"),
            ColumnType::BigInt => write!(f, "BIGINT"),
            ColumnType::Int128 => write!(f, "DECIMAL"),
            ColumnType::Decimal75(precision, scale) => {
                write!(
                    f,
                    "DECIMAL75(PRECISION: {:?}, SCALE: {scale})",
                    precision.value()
                )
            }
            ColumnType::VarChar => write!(f, "VARCHAR"),
            ColumnType::VarBinary => write!(f, "BINARY"),
            ColumnType::Scalar => write!(f, "SCALAR"),
            ColumnType::TimestampTZ(timeunit, timezone) => {
                write!(f, "TIMESTAMP(TIMEUNIT: {timeunit}, TIMEZONE: {timezone})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"TimestampTZ":["Second",{"offset":0}]}"#);

        let column_type = ColumnType::Boolean;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Boolean""#);

        let column_type = ColumnType::TinyInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""TinyInt""#);

        let column_type = ColumnType::SmallInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""SmallInt""#);

        let column_type = ColumnType::Int;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Int""#);

        let column_type = ColumnType::BigInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""BigInt""#);

        let column_type = ColumnType::Int128;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Decimal""#);

        let column_type = ColumnType::VarChar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""VarChar""#);

        let column_type = ColumnType::Scalar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Scalar""#);

        let column_type = ColumnType::Decimal75(Precision::new(1).unwrap(), 0);
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#"{"Decimal75":[1,0]}"#);
    }

    #[test]
    fn we_can_deserialize_columns_from_valid_strings() {
        let expected_column_type =
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());
        let deserialized: ColumnType =
            serde_json::from_str(r#"{"TimestampTZ":["Second",{"offset":0}]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Boolean;
        let deserialized: ColumnType = serde_json::from_str(r#""Boolean""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::TinyInt;
        let deserialized: ColumnType = serde_json::from_str(r#""TinyInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::SmallInt;
        let deserialized: ColumnType = serde_json::from_str(r#""SmallInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int;
        let deserialized: ColumnType = serde_json::from_str(r#""Int""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::BigInt;
        let deserialized: ColumnType = serde_json::from_str(r#""BigInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::TinyInt;
        let deserialized: ColumnType = serde_json::from_str(r#""TINYINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::SmallInt;
        let deserialized: ColumnType = serde_json::from_str(r#""SMALLINT""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""DECIMAL""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""Decimal""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::VarChar;
        let deserialized: ColumnType = serde_json::from_str(r#""VarChar""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Scalar;
        let deserialized: ColumnType = serde_json::from_str(r#""SCALAR""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), i8::MAX);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[75, 127]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Decimal75(Precision::new(1).unwrap(), i8::MIN);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[1, -128]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Decimal75(Precision::new(1).unwrap(), 0);
        let deserialized: ColumnType = serde_json::from_str(r#"{"Decimal75":[1, 0]}"#).unwrap();
        assert_eq!(deserialized, expected_column_type);
    }

    #[test]
    fn we_can_deserialize_columns_from_lowercase_or_uppercase_strings() {
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""boolean""#).unwrap(),
            ColumnType::Boolean
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BOOLEAN""#).unwrap(),
            ColumnType::Boolean
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""bigint""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BIGINT""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""TINYINT""#).unwrap(),
            ColumnType::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""tinyint""#).unwrap(),
            ColumnType::TinyInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""SMALLINT""#).unwrap(),
            ColumnType::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""smallint""#).unwrap(),
            ColumnType::SmallInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""int""#).unwrap(),
            ColumnType::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""INT""#).unwrap(),
            ColumnType::Int
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""decimal""#).unwrap(),
            ColumnType::Int128
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""DECIMAL""#).unwrap(),
            ColumnType::Int128
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""VARCHAR""#).unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""varchar""#).unwrap(),
            ColumnType::VarChar
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""SCALAR""#).unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""scalar""#).unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"decimal75":[1,0]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"DECIMAL75":[1,0]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"decimal75":[10,5]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(10).unwrap(), 5)
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#"{"DECIMAL75":[1,-128]}"#).unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), -128)
        );
    }

    #[test]
    fn we_cannot_deserialize_columns_from_invalid_strings() {
        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""BooLean""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Tinyint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Smallint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""iNt""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Bigint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""DecImal""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""DecImal75""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> =
            serde_json::from_str(r#"{"TimestampTZ":["Utc","Second"]}"#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Varchar""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""ScaLar""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn we_can_convert_columntype_to_json_string_and_back() {
        let boolean = ColumnType::Boolean;
        let boolean_json = serde_json::to_string(&boolean).unwrap();
        assert_eq!(boolean_json, "\"Boolean\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&boolean_json).unwrap(),
            boolean
        );

        let tinyint = ColumnType::TinyInt;
        let tinyint_json = serde_json::to_string(&tinyint).unwrap();
        assert_eq!(tinyint_json, "\"TinyInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&tinyint_json).unwrap(),
            tinyint
        );

        let smallint = ColumnType::SmallInt;
        let smallint_json = serde_json::to_string(&smallint).unwrap();
        assert_eq!(smallint_json, "\"SmallInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&smallint_json).unwrap(),
            smallint
        );

        let int = ColumnType::Int;
        let int_json = serde_json::to_string(&int).unwrap();
        assert_eq!(int_json, "\"Int\"");
        assert_eq!(serde_json::from_str::<ColumnType>(&int_json).unwrap(), int);

        let bigint = ColumnType::BigInt;
        let bigint_json = serde_json::to_string(&bigint).unwrap();
        assert_eq!(bigint_json, "\"BigInt\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&bigint_json).unwrap(),
            bigint
        );

        let int128 = ColumnType::Int128;
        let int128_json = serde_json::to_string(&int128).unwrap();
        assert_eq!(int128_json, "\"Decimal\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&int128_json).unwrap(),
            int128
        );

        let varchar = ColumnType::VarChar;
        let varchar_json = serde_json::to_string(&varchar).unwrap();
        assert_eq!(varchar_json, "\"VarChar\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&varchar_json).unwrap(),
            varchar
        );

        let scalar = ColumnType::Scalar;
        let scalar_json = serde_json::to_string(&scalar).unwrap();
        assert_eq!(scalar_json, "\"Scalar\"");
        assert_eq!(
            serde_json::from_str::<ColumnType>(&scalar_json).unwrap(),
            scalar
        );

        let decimal75 = ColumnType::Decimal75(Precision::new(75).unwrap(), 0);
        let decimal75_json = serde_json::to_string(&decimal75).unwrap();
        assert_eq!(decimal75_json, r#"{"Decimal75":[75,0]}"#);
        assert_eq!(
            serde_json::from_str::<ColumnType>(&decimal75_json).unwrap(),
            decimal75
        );
    }

    #[test]
    fn we_can_get_min_scalar() {
        assert_eq!(
            ColumnType::TinyInt.min_scalar(),
            Some(TestScalar::from(i8::MIN))
        );
        assert_eq!(
            ColumnType::SmallInt.min_scalar(),
            Some(TestScalar::from(i16::MIN))
        );
        assert_eq!(
            ColumnType::Int.min_scalar(),
            Some(TestScalar::from(i32::MIN))
        );
        assert_eq!(
            ColumnType::BigInt.min_scalar(),
            Some(TestScalar::from(i64::MIN))
        );
        assert_eq!(
            ColumnType::Int128.min_scalar(),
            Some(TestScalar::from(i128::MIN))
        );
        assert_eq!(ColumnType::Uint8.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::Scalar.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::Boolean.min_scalar::<TestScalar>(), None);
        assert_eq!(ColumnType::VarBinary.min_scalar::<TestScalar>(), None);
        assert_eq!(
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(0))
                .min_scalar::<TestScalar>(),
            None
        );
        assert_eq!(
            ColumnType::Decimal75(Precision::new(1).unwrap(), 1).min_scalar::<TestScalar>(),
            None
        );
        assert_eq!(ColumnType::VarChar.min_scalar::<TestScalar>(), None);
    }

    #[test]
    fn we_can_get_sqrt_negative_min() {
        for column_type in [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            let floor = TestScalar::from(column_type.sqrt_negative_min().unwrap());
            let ceiling = floor + TestScalar::ONE;
            let floor_squared = floor * floor;
            let ceiling_squared = ceiling * ceiling;
            let negative_min_scalar = -column_type.min_scalar::<TestScalar>().unwrap();
            assert!(floor_squared <= negative_min_scalar);
            assert!(negative_min_scalar < ceiling_squared);
        }
        for column_type in [
            ColumnType::Uint8,
            ColumnType::Scalar,
            ColumnType::Boolean,
            ColumnType::VarBinary,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(1)),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 1),
            ColumnType::VarChar,
        ] {
            assert_eq!(column_type.sqrt_negative_min(), None);
        }
    }
}
