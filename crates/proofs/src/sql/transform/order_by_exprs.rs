use super::DataFrameExpr;
use crate::base::database::{INT128_PRECISION, INT128_SCALE};

use arrow::datatypes::ArrowNativeTypeOp;
use proofs_sql::intermediate_ast::{OrderBy, OrderByDirection};

use dyn_partial_eq::DynPartialEq;
use polars::prelude::{col, DataType, Expr, GetOutput, LazyFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};

/// A node representing a list of `OrderBy` expressions.
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct OrderByExprs {
    by_exprs: Vec<OrderBy>,
}

impl OrderByExprs {
    /// Create a new `OrderByExprs` node.
    pub fn new(by_exprs: Vec<OrderBy>) -> Self {
        Self { by_exprs }
    }
}

#[typetag::serde]
impl DataFrameExpr for OrderByExprs {
    /// Sort the `LazyFrame` by the `OrderBy` expressions.
    fn apply_transformation(&self, lazy_frame: LazyFrame, _: usize) -> LazyFrame {
        assert!(!self.by_exprs.is_empty());

        let maintain_order = true;
        let nulls_last = false;
        let reverse: Vec<_> = self
            .by_exprs
            .iter()
            .map(|v| v.direction == OrderByDirection::Desc)
            .collect();
        let by_column: Vec<_> = self
            .by_exprs
            .iter()
            .map(|v| order_by_map_to_utf8_if_decimal(col(v.expr.name())))
            .collect();

        lazy_frame.sort_by_exprs(by_column, reverse, nulls_last, maintain_order)
    }
}

/// Converts a signed 128-bit integer to a UTF-8 string that preserves
/// the order of the original integer array when sorted.
///
/// For any given two integers `a` and `b` we have:
/// * `a < b` if and only if `map_i128_to_utf8(a) < map_i128_to_utf8(b)`.
/// * `a == b` if and only if `map_i128_to_utf8(a) == map_i128_to_utf8(b)`.
/// * `a > b` if and only if `map_i128_to_utf8(a) > map_i128_to_utf8(b)`.
pub(crate) fn order_by_map_i128_to_utf8(v: i128) -> String {
    let is_neg = v.is_negative() as u8;
    v.abs()
        // use big-endian order to allow skipping the leading zero bytes
        .to_be_bytes()
        .into_iter()
        // skip the leading zero bytes to save space
        .skip_while(|c| c.is_zero())
        .collect::<Vec<_>>()
        .into_iter()
        // reverse back to little-endian order
        .rev()
        // append a byte that indicates the number of leading zero bits
        // this is necessary because "12" is lexicographically smaller than "9"
        // which is not the case for the original integer array as 9 < 12.
        // so we append the number of leading zero bits to guarantee that "{byte}9" < "{byte}12"
        .chain(std::iter::once((255 - v.abs().leading_zeros()) as u8 + 1))
        // transform the bytes of negative values so that smaller negative numbers converted
        // to strings can appear before larger negative numbers converted to strings
        .map(|c| (255 - c) * is_neg + c * (1 - is_neg))
        .map(char::from)
        .rev()
        .collect()
}

// Polars doesn't support Decimal columns inside order by.
// So we need to remap them to the supported UTF8 type.
fn order_by_map_to_utf8_if_decimal(expr: Expr) -> Expr {
    expr.map(
        |series| match series.dtype().clone() {
            DataType::Decimal(Some(INT128_PRECISION), Some(INT128_SCALE)) => {
                let i128_data = series.decimal().unwrap().into_no_null_iter();
                // TODO: remove this mapping once Polars supports decimal columns inside order by
                // Issue created to track progress: https://github.com/pola-rs/polars/issues/11079
                let utf8_data = i128_data.map(order_by_map_i128_to_utf8).collect::<Vec<_>>();
                Ok(Some(Series::new(series.name(), utf8_data)))
            }
            _ => Ok(Some(series)),
        },
        GetOutput::from_type(DataType::Utf8),
    )
}
