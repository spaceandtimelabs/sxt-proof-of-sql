use crate::base::{database::ColumnType, math::decimal::Precision, scalar::Scalar};
use alloc::string::String;
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use serde::{Deserialize, Serialize};

/// Represents a literal value.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LiteralValue<S: Scalar> {
    /// Boolean literals
    Boolean(bool),
    /// i8 literals
    TinyInt(i8),
    /// i16 literals
    SmallInt(i16),
    /// i32 literals
    Int(i32),
    /// i64 literals
    BigInt(i64),

    /// String literals
    ///  - the first element maps to the str value.
    ///  - the second element maps to the str hash (see [`crate::base::scalar::Scalar`]).
    VarChar((String, S)),
    /// i128 literals
    Int128(i128),
    /// Decimal literals with a max width of 252 bits
    ///  - the backing store maps to the type [`crate::base::scalar::Curve25519Scalar`]
    Decimal75(Precision, i8, S),
    /// Scalar literals
    Scalar(S),
    /// `TimeStamp` defined over a unit (s, ms, ns, etc) and timezone with backing store
    /// mapped to i64, which is time units since unix epoch
    TimeStampTZ(PoSQLTimeUnit, PoSQLTimeZone, i64),
}

impl<S: Scalar> LiteralValue<S> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::TinyInt(_) => ColumnType::TinyInt,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::TimeStampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }

    /// Converts the literal to a scalar
    pub(crate) fn to_scalar(&self) -> S {
        match self {
            Self::Boolean(b) => b.into(),
            Self::TinyInt(i) => i.into(),
            Self::SmallInt(i) => i.into(),
            Self::Int(i) => i.into(),
            Self::BigInt(i) => i.into(),
            Self::VarChar((_, s)) | Self::Decimal75(_, _, s) => *s,
            Self::Int128(i) => i.into(),
            Self::Scalar(scalar) => *scalar,
            Self::TimeStampTZ(_, _, time) => time.into(),
        }
    }
}
