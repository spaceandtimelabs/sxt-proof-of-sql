use polars::prelude::{DataType, Expr, Literal, Series};

use crate::base::database::{INT128_PRECISION, INT128_SCALE};

pub trait LiteralConversion {
    fn to_lit(&self) -> Expr;
}

impl LiteralConversion for i128 {
    fn to_lit(&self) -> Expr {
        let s = [self.abs().to_string()].into_iter().collect::<Series>();
        let l = s.lit().cast(DataType::Decimal(
            Some(INT128_PRECISION),
            Some(INT128_SCALE),
        ));

        if self.is_negative() {
            [-1].into_iter().collect::<Series>().lit() * l
        } else {
            l
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record_batch as batch;
    use crate::sql::proof::TransformExpr;
    use crate::sql::transform::{test_utility::select, ResultExpr};

    const MAX_I64: i128 = i64::MAX as i128;
    const MIN_I64: i128 = i64::MIN as i128;
    const MAX_DECIMAL: i128 = 10_i128.pow(38) - 1;
    const MIN_DECIMAL: i128 = -(10_i128.pow(38) - 1);

    macro_rules! test_expr {
        ($expr:expr, $expected:expr) => {
            let data = batch!("" => [0_i64]);
            let result = ResultExpr::new(select(&[$expr])).transform_results(data);
            assert_eq!(result, $expected);
        };
        ($expr:expr, $expected:expr, $data:expr) => {
            assert_eq!(ResultExpr::new(select(&[$expr.alias("")])).transform_results($data), $expected);
        };
    }

    #[test]
    fn i128_can_be_properly_converted_to_lit() {
        test_expr! {1_i128.to_lit(), batch!("" => [1_i128])};
        test_expr! {0_i128.to_lit(), batch!("" => [0_i128])};
        test_expr! {(-1_i128).to_lit(), batch!("" => [-1_i128])};
        test_expr! {MAX_DECIMAL.to_lit(), batch!("" => [MAX_DECIMAL])};
        test_expr! {(MIN_DECIMAL).to_lit(), batch!("" => [MIN_DECIMAL])};
        test_expr! {(MIN_DECIMAL + 1).to_lit(), batch!("" => [MIN_DECIMAL + 1])};
        test_expr! {(MAX_DECIMAL - 1).to_lit(), batch!("" => [MAX_DECIMAL - 1])};
        test_expr!(MAX_I64.to_lit(), batch!("" => [i64::MAX as i128]));
        test_expr!(MIN_I64.to_lit(), batch!("" => [i64::MIN as i128]));
        (-3000..3000).for_each(|i| {
            test_expr! {i.to_lit(), batch!("" => [i])};
        });
    }
}
