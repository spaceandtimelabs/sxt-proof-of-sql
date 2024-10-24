use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray};
use arrow::compute::{cast_with_options, CastOptions};
use arrow::datatypes::DataType as ArrowDataType;
use arrow::error::ArrowError;
use arrow::util::display::FormatOptions;
use bigdecimal::{BigDecimal, ParseBigDecimalError};
use snafu::Snafu;
use sqlparser::ast::{DataType as SqlparserDataType, ExactNumberInfo};

/// Errors that can occur when parsing string columns to decimal columns.
#[derive(Debug, Snafu)]
pub enum ParseDecimalsError {
    /// Unable to parse string value to BigDecimal.
    #[snafu(display("unable to parse string value to BigDecimal: {error}"))]
    BigDecimal {
        /// The source bigdecimal error.
        error: ParseBigDecimalError,
    },

    /// Unable to cast string value to decimal256.
    #[snafu(display("unable to cast string value to Decimal256: {error}"))]
    Cast {
        /// The source decimal256 error.
        error: ArrowError,
    },
}

impl From<ParseBigDecimalError> for ParseDecimalsError {
    fn from(error: ParseBigDecimalError) -> Self {
        ParseDecimalsError::BigDecimal { error }
    }
}

impl From<ArrowError> for ParseDecimalsError {
    fn from(error: ArrowError) -> Self {
        ParseDecimalsError::Cast { error }
    }
}

/// Returns the provided column with strings parsed to decimals if the column type is string and
/// the target type is decimal.
///
/// Errors if the cast fails.
pub fn column_parse_decimals_fallible(
    column: ArrayRef,
    target_type: &SqlparserDataType,
) -> Result<ArrayRef, ParseDecimalsError> {
    match (column.data_type(), target_type) {
        (
            ArrowDataType::Utf8,
            SqlparserDataType::Numeric(number_info)
            | SqlparserDataType::Decimal(number_info)
            | SqlparserDataType::BigNumeric(number_info)
            | SqlparserDataType::BigDecimal(number_info)
            | SqlparserDataType::Dec(number_info),
        ) => {
            let (precision, scale) = match number_info {
                ExactNumberInfo::None => (75, 0),
                ExactNumberInfo::Precision(p) => ((*p as u8).min(75), 0),
                ExactNumberInfo::PrecisionAndScale(p, s) => {
                    ((*p as u8).min(75), *s as i8)
                }
            };

            // bigdecimal can parse scientific notation
            //
            // Parsing w/ both bigdecimal then casting w/ arrow is a bit redundant.
            // However, we've had issues trying to convert from bigdecimals to arrow i256 before.
            let column: ArrayRef = Arc::new(StringArray::from_iter(
                column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .map(|maybe_string| {
                        maybe_string
                            .map(|string| {
                                BigDecimal::from_str(string).map(|decimal| decimal.to_string())
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, ParseBigDecimalError>>()?,
            ));

            // Casting to p+1 avoids an arrow error that was only recently fixed (not released)
            // https://github.com/apache/arrow-rs/issues/5876
            let column = cast_with_options(
                &column,
                &ArrowDataType::Decimal256(precision + 1, scale + 1),
                &CastOptions {
                    safe: false,
                    format_options: FormatOptions::new(),
                },
            )?;
            Ok(cast_with_options(
                &column,
                &ArrowDataType::Decimal256(precision, scale),
                &CastOptions {
                    safe: false,
                    format_options: FormatOptions::new(),
                },
            )?)
        }
        _ => Ok(column),
    }
}

/// Returns the provided column with strings parsed to decimals if the column type is string and
/// the target type is decimal.
///
/// Panics if the cast fails.
pub fn column_parse_decimals_unchecked(
    column: ArrayRef,
    target_type: &SqlparserDataType,
) -> ArrayRef {
    column_parse_decimals_fallible(column, target_type)
        .expect("string column unable to parse to decimals")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::array::{Decimal256Array, StringArray};
    use arrow::datatypes::i256;

    use super::*;

    #[test]
    fn we_can_parse_decimals() {
        let max_number = "9".repeat(75);
        let mut min_number = max_number.clone();
        min_number.insert(0, '-');
        let column: ArrayRef = Arc::new(StringArray::from_iter_values([
            "0",
            &max_number,
            &min_number,
        ]));

        let data_type = SqlparserDataType::Numeric(ExactNumberInfo::PrecisionAndScale(75, 0));

        let expected: ArrayRef = Arc::new(
            Decimal256Array::from_iter_values([
                i256::from_i128(0),
                i256::from_str(&max_number).unwrap(),
                i256::from_str(&min_number).unwrap(),
            ])
            .with_precision_and_scale(75, 0)
            .unwrap(),
        );

        assert_eq!(
            &column_parse_decimals_unchecked(column, &data_type),
            &expected
        );

        let column: ArrayRef = Arc::new(StringArray::from_iter_values(["0", "-10.5", "2e4"]));

        let data_type = SqlparserDataType::Decimal(ExactNumberInfo::PrecisionAndScale(10, 2));

        let expected: ArrayRef = Arc::new(
            Decimal256Array::from_iter_values([
                i256::from_i128(0),
                i256::from_i128(-1050),
                i256::from_i128(2000000),
            ])
            .with_precision_and_scale(10, 2)
            .unwrap(),
        );

        assert_eq!(
            &column_parse_decimals_unchecked(column, &data_type),
            &expected
        );
    }

    #[test]
    fn we_cannot_parse_nondecimals() {
        let column: ArrayRef =
            Arc::new(StringArray::from_iter_values(["0", "not a decimal", "200"]));

        let data_type = SqlparserDataType::Decimal(ExactNumberInfo::PrecisionAndScale(75, 0));
        assert!(matches!(
            column_parse_decimals_fallible(column, &data_type),
            Err(ParseDecimalsError::BigDecimal { .. })
        ))
    }

    #[test]
    fn we_cannot_parse_out_of_bounds_decimals() {
        let excessive_precision = "9".repeat(76);
        let column: ArrayRef = Arc::new(StringArray::from_iter_values([&excessive_precision]));

        let data_type = SqlparserDataType::Numeric(ExactNumberInfo::PrecisionAndScale(75, 0));
        assert!(matches!(
            column_parse_decimals_fallible(column, &data_type),
            Err(ParseDecimalsError::Cast { .. })
        ));
    }
}
