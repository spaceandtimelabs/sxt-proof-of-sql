#[cfg(feature = "std")]
use alloc::sync::Arc;

#[cfg(feature = "std")]
use arrow::array::{ArrayRef, Decimal256Array};
#[cfg(feature = "std")]
use arrow::datatypes::DataType as ArrowDataType;
use sqlparser::ast::{ColumnDef, DataType as SqlparserDataType, ExactNumberInfo};

/// Maximum decimal precision supported by proof of sql.
pub const MAX_PRECISION: u8 = 75;

/// Returns the provided number info with precision clamped to the proof of sql maximum.
fn number_info_clamp_precision(number_info: ExactNumberInfo) -> ExactNumberInfo {
    match number_info {
        ExactNumberInfo::None => ExactNumberInfo::Precision(MAX_PRECISION as u64),
        ExactNumberInfo::Precision(p) => ExactNumberInfo::Precision(p.min(MAX_PRECISION as u64)),
        ExactNumberInfo::PrecisionAndScale(p, s) => {
            ExactNumberInfo::PrecisionAndScale(p.min(MAX_PRECISION as u64), s)
        }
    }
}

/// Returns the provided column def with precision clamped to the proof of sql maximum if the
/// column def is a decimal.
pub fn column_def_clamp_precision(column: ColumnDef) -> ColumnDef {
    let data_type = match column.data_type {
        SqlparserDataType::Numeric(number_info) => {
            SqlparserDataType::Numeric(number_info_clamp_precision(number_info))
        }
        SqlparserDataType::Decimal(number_info) => {
            SqlparserDataType::Decimal(number_info_clamp_precision(number_info))
        }
        SqlparserDataType::BigNumeric(number_info) => {
            SqlparserDataType::BigNumeric(number_info_clamp_precision(number_info))
        }
        SqlparserDataType::BigDecimal(number_info) => {
            SqlparserDataType::BigDecimal(number_info_clamp_precision(number_info))
        }
        SqlparserDataType::Dec(number_info) => {
            SqlparserDataType::Dec(number_info_clamp_precision(number_info))
        }
        data_type => data_type,
    };

    ColumnDef {
        data_type,
        ..column
    }
}

/// Returns the provided column with precision clamped to the proof of sql maximum if the column
/// is Decimal256.
#[cfg(feature = "std")]
pub fn column_clamp_precision(column: ArrayRef) -> ArrayRef {
    match column.data_type() {
        ArrowDataType::Decimal256(precision, scale) if precision > &MAX_PRECISION => Arc::new(
            column
                .as_any()
                .downcast_ref::<Decimal256Array>()
                .unwrap()
                .clone()
                .with_precision_and_scale(MAX_PRECISION, *scale)
                .expect("this error is exceedingly unlikely, only occurs if the scale of the source column is 76"),
        ),
        _ => column,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use sqlparser::ast::Ident;

    use super::*;

    #[test]
    fn we_can_clamp_decimal_column_def() {
        let nullable_column = ColumnDef {
            name: Ident::new("numeric_col"),
            data_type: sqlparser::ast::DataType::Numeric(ExactNumberInfo::None),
            collation: None,
            options: vec![],
        };
        let expected = ColumnDef {
            name: Ident::new("numeric_col"),
            data_type: sqlparser::ast::DataType::Numeric(ExactNumberInfo::Precision(75)),
            collation: None,
            options: vec![],
        };

        assert_eq!(&column_def_clamp_precision(nullable_column), &expected);
        assert_eq!(&column_def_clamp_precision(expected.clone()), &expected);

        let nullable_column = ColumnDef {
            name: Ident::new("dec_col"),
            data_type: sqlparser::ast::DataType::Dec(ExactNumberInfo::Precision(78)),
            collation: None,
            options: vec![],
        };

        let expected = ColumnDef {
            name: Ident::new("dec_col"),
            data_type: sqlparser::ast::DataType::Dec(ExactNumberInfo::Precision(75)),
            collation: None,
            options: vec![],
        };

        assert_eq!(&column_def_clamp_precision(nullable_column), &expected);
        assert_eq!(&column_def_clamp_precision(expected.clone()), &expected);

        let nullable_column = ColumnDef {
            name: Ident::new("decimal_col"),
            data_type: sqlparser::ast::DataType::Decimal(ExactNumberInfo::PrecisionAndScale(78, 5)),
            collation: None,
            options: vec![],
        };

        let expected = ColumnDef {
            name: Ident::new("decimal_col"),
            data_type: sqlparser::ast::DataType::Decimal(ExactNumberInfo::PrecisionAndScale(75, 5)),
            collation: None,
            options: vec![],
        };

        assert_eq!(&column_def_clamp_precision(nullable_column), &expected);
        assert_eq!(&column_def_clamp_precision(expected.clone()), &expected);
    }
}

#[cfg(all(test, feature = "std"))]
mod std_tests {
    use arrow::datatypes::i256;

    use super::*;

    #[test]
    fn we_can_clamp_decimal_columns() {
        let column: ArrayRef = Arc::new(
            Decimal256Array::from_iter_values([0, 100, -10000].map(i256::from))
                .with_precision_and_scale(76, 5)
                .unwrap(),
        );

        let result = column_clamp_precision(column);
        let expected: ArrayRef = Arc::new(
            Decimal256Array::from_iter_values([0, 100, -10000].map(i256::from))
                .with_precision_and_scale(75, 5)
                .unwrap(),
        );

        assert_eq!(&result, &expected);
    }
}
