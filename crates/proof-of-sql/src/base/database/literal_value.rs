use crate::{
    alloc::string::ToString,
    base::{
        database::ColumnType,
        math::{decimal::Precision, i256::I256},
        scalar::Scalar,
    },
};
use alloc::vec::Vec;
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use sqlparser::ast::{DataType, ExactNumberInfo, Expr, Value};

/// A trait for SQL expressions that provides functionality to retrieve their associated column type.
/// This trait is primarily used to map SQL expressions to their corresponding [`ColumnType`].
pub trait ExprExt {
    /// Determines the [`ColumnType`] associated with the expression.
    fn column_type(&self) -> ColumnType;
}

/// A trait for SQL expressions that allows converting them into scalar values.
/// This trait provides functionality to interpret SQL expressions as scalars
pub trait ToScalar {
    /// Converts the SQL expression into a scalar value of the specified type.
    fn to_scalar<S: Scalar>(&self) -> S;
}

impl ExprExt for Expr {
    /// Provides the column type associated with the column
    #[must_use]
    fn column_type(&self) -> ColumnType {
        match self {
            Expr::Value(Value::Boolean(_)) => ColumnType::Boolean,
            Expr::Value(Value::Number(value, _)) => {
                let n = value.parse::<i128>().unwrap_or_else(|err| {
                    panic!("Failed to parse '{value}' as a number. Error: {err}");
                });
                if i8::try_from(n).is_ok() {
                    ColumnType::TinyInt
                } else if i16::try_from(n).is_ok() {
                    ColumnType::SmallInt
                } else if i32::try_from(n).is_ok() {
                    ColumnType::Int
                } else if i64::try_from(n).is_ok() {
                    ColumnType::BigInt
                } else {
                    ColumnType::Int128
                }
            }
            Expr::Value(Value::SingleQuotedString(_)) => ColumnType::VarChar,
            Expr::TypedString { data_type, .. } => match data_type {
                DataType::Decimal(ExactNumberInfo::PrecisionAndScale(p, s)) => {
                    let precision = u8::try_from(*p).expect("Precision must fit into u8");
                    let scale = i8::try_from(*s).expect("Scale must fit into i8");
                    let precision_obj =
                        Precision::new(precision).expect("Failed to create Precision");
                    ColumnType::Decimal75(precision_obj, scale)
                }
                DataType::Timestamp(Some(precision), tz) => {
                    let tu =
                        PoSQLTimeUnit::from_precision(*precision).unwrap_or(PoSQLTimeUnit::Second);
                    ColumnType::TimestampTZ(tu, *tz)
                }
                DataType::Custom(_, _) if data_type.to_string() == "scalar" => ColumnType::Scalar,
                _ => unimplemented!("Mapping for {:?} is not implemented", data_type),
            },
            _ => unimplemented!("Mapping for {:?} is not implemented", self),
        }
    }
}

impl ToScalar for Expr {
    /// Converts the literal to a scalar
    fn to_scalar<S: Scalar>(&self) -> S {
        match self {
            Expr::Value(Value::Boolean(b)) => b.into(),
            Expr::Value(Value::Number(n, _)) => n
                .parse::<i128>()
                .unwrap_or_else(|_| panic!("Invalid number: {n}"))
                .into(),
            Expr::Value(Value::SingleQuotedString(s)) => s.into(),
            Expr::TypedString { data_type, value } if data_type.to_string() == "scalar" => {
                let scalar_str = value.strip_prefix("scalar:").unwrap();
                let limbs: Vec<u64> = scalar_str
                    .split(',')
                    .map(|x| x.parse::<u64>().unwrap())
                    .collect();
                assert!(limbs.len() == 4, "Scalar must have exactly 4 limbs");
                S::from([limbs[0], limbs[1], limbs[2], limbs[3]])
            }
            Expr::TypedString { data_type, value } => match data_type {
                DataType::Timestamp(_, _) => value.parse::<i64>().unwrap().into(),
                DataType::Decimal(_) => {
                    let i256_value = I256::from_string(value)
                        .unwrap_or_else(|_| panic!("Failed to parse '{value}' as a decimal"));
                    i256_value.into_scalar()
                }
                _ => unimplemented!("Conversion for {:?} is not implemented.", data_type),
            },
            _ => unimplemented!("Conversion for {:?} is not implemented", self),
        }
    }
}
