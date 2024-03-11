use serde::{Deserialize, Serialize};

/// An intermediate placeholder for a decimal string
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DecimalUnknown {
    value: String,
    precision: u8,
    pub scale: i8,
}

impl DecimalUnknown {
    /// A parser conforming to standard postgreSQL to parse the precision and scale
    /// from a decimal token obtained from the lalrpop lexer. This constructor
    /// serves as an intermediate type to avoid cyclic dependency between proofs
    /// and proofs-sql. It observes the following special cases:
    ///     
    /// Leading and trailing zeros are not counted towards precision.
    ///
    /// Arbitrary precision is supported here. A precision exceeding the
    /// maximum supported precision in proofs is handled fallibly in the
    /// proofs crate.
    ///
    /// A decimal must have a decimal point. The lexer does not route
    /// whole integers to this contructor.
    /// The purpose and objective of this function is to parse a decimal
    /// string into a fixed-point representation.
    pub fn new(decimal_string: &str) -> Self {
        // Split the value into integer and fractional parts
        let parts: Vec<&str> = decimal_string.split('.').collect();
        let integer_part = parts[0];
        let mut fractional_part = parts.get(1).unwrap_or(&"").to_string();

        let mut value = decimal_string.to_owned();

        // Conditionally trim trailing zeros from the fractional part
        if fractional_part.ends_with('0') && fractional_part.len() > 1 {
            // Trim all trailing zeros if more than one
            while fractional_part.ends_with('0') {
                fractional_part.pop();
            }
            // Reconstruct value if fractional part was modified
            value = if fractional_part.is_empty() {
                integer_part.to_string()
            } else {
                format!("{}.{}", integer_part, fractional_part)
            };
        }

        // Remove any leading + or - signs for the purpose of calculating precision
        let value_without_sign = value.trim_start_matches(|c: char| c == '+' || c == '-');

        let parts: Vec<&str> = value_without_sign.split('.').collect();
        // Remove leading zeros from the integer part for precision calculation

        // Only trim leading zeros if there are more than 1 of them i.e.
        // trim 00.1 -> 0.1 and 00.0 -> 0.0
        let integer_part_digits = if parts[0].starts_with('0') && parts[0].len() > 1 {
            parts[0].trim_start_matches('0')
        } else {
            parts[0]
        }
        .len();

        // Scale is the length of the fractional part after trimming
        let scale = fractional_part.len() as i8;

        // Total precision is the sum of integer part digits and scale
        let precision = (integer_part_digits as i8 + scale) as u8;

        // Interpret as integer
        let value = decimal_string
            .trim_end_matches('0')
            .trim_end_matches('.')
            .replace('.', "");

        DecimalUnknown {
            value,
            precision,
            scale,
        }
    }

    pub fn value(&self) -> String {
        self.value.to_owned()
    }
    pub fn precision(&self) -> u8 {
        self.precision
    }
    pub fn scale(&self) -> i8 {
        self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_parsing() {
        let cases = vec![
            ("0.0", "0", 2, 1),
            ("-0.456", "-0456", 4, 3),
            ("0.001", "0001", 4, 3),
            ("123.456", "123456", 6, 3),
            ("-123.456", "-123456", 6, 3),
            (".456", "456", 3, 3),
            ("123.", "123", 3, 0),
            ("123456789.987654321", "123456789987654321", 18, 9),
            (".123456789", "123456789", 9, 9),
            // this should be ok for now because this
            // type has no expectations about p/s
            (
                "3618502788666131106986593281521497120428.558179689953803000975469142727125494",
                "3618502788666131106986593281521497120428558179689953803000975469142727125494",
                76,
                36,
            ),
        ];
        for (input, expected_value, expected_precision, expected_scale) in cases {
            let decimal = DecimalUnknown::new(input);
            assert_eq!(decimal.value, expected_value);
            assert_eq!(decimal.precision, expected_precision);
            assert_eq!(decimal.scale, expected_scale);
        }
    }
}
