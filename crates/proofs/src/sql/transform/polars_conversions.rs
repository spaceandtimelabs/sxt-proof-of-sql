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

    use polars::prelude::col;
    use rand::distributions::Uniform;
    use rand::Rng;

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

    macro_rules! safe_arithmetic {
        ($op:expr, $x:expr, $y:expr, $x_e:expr, $y_e:expr) => {
            let data = batch!("x" => [$x], "y" => [$y]);

            match $op {
                "add" => {
                    if $x.checked_add($y).is_some() && ($x + $y) <= MAX_DECIMAL && ($x + $y) >= MIN_DECIMAL {
                        test_expr!($x_e + $y_e, batch!("" => [$x + $y]), data);
                    }
                }
                "sub" => {
                    if $x.checked_sub($y).is_some() && ($x - $y) <= MAX_DECIMAL && ($x - $y) >= MIN_DECIMAL {
                        test_expr!($x.to_lit() - $y.to_lit(), batch!("" => [$x - $y]), data);
                    }
                }
                "mul" => {
                    if $x.checked_mul($y).is_some() && ($x * $y) <= MAX_DECIMAL && ($x * $y) >= MIN_DECIMAL {
                        test_expr!($x.to_lit() * $y.to_lit(), batch!("" => [$x * $y]), data);
                    }
                }
                "div" => {
                    if $y != 0 {
                        test_expr!($x.to_lit() / $y.to_lit(), batch!("" => [$x / $y]), data);
                    }
                }
                _ => panic!("Invalid operation"),
            }
        };
    }

    macro_rules! batch_execute_test {
        ($batch:expr) => {
            for [x, y] in $batch {
                for [x, y] in [[x, y], [y, x]] {
                    for op in ["add", "sub", "mul", "div"].into_iter() {
                        safe_arithmetic!(op, x, y, x.to_lit(), y.to_lit());
                        safe_arithmetic!(op, x, y, x.to_lit(), col("y"));
                        safe_arithmetic!(op, x, y, col("x"), y.to_lit());
                        safe_arithmetic!(op, x, y, col("x"), col("y"));

                        ///////////////////////////////////////////////////////////////////////////////
                        // TODO: Address Precision Loss between decimal and i64 columns
                        ///////////////////////////////////////////////////////////////////////////////
                        // The following tests encounter issues due to the automatic
                        // casting of i64 to f64 in Polars, resulting in precision loss.
                        // A fix has been proposed in this pull request:
                        // https://github.com/pola-rs/polars/pull/11166.
                        //
                        // However, since the merge may take time,
                        // I  plan to implement a workaround in a subsequent pull request.
                        // This workaround involves explicit casting to decimal(38, 0)
                        // when i64 columns are utilized. This will work.
                        ///////////////////////////////////////////////////////////////////////////////
                        // if x >= i64::MIN as i128 && x <= i64::MAX as i128 {
                        //     safe_arithmetic!(op, x, y, col("x").cast(DataType::Int64), col("y"));
                        //     safe_arithmetic!(op, x, y, col("x").cast(DataType::Int64), y.to_lit());
                        // }
                        ///////////////////////////////////////////////////////////////////////////////
                        // if i64::try_from(x).is_ok() {
                        //     safe_arithmetic!(op, x, y, col("x").cast(DataType::Int64), y.to_lit());
                        //     safe_arithmetic!(op, x, y, col("x").cast(DataType::Int64), col("y"));
                        // }
                        ///////////////////////////////////////////////////////////////////////////////
                        // if i64::try_from(y).is_ok() {
                        //     safe_arithmetic!(op, x, y, col("x"), col("y").cast(DataType::Int64));
                        //     safe_arithmetic!(op, x, y, x.to_lit(), col("y").cast(DataType::Int64));
                        // }
                        ///////////////////////////////////////////////////////////////////////////////
                        // if i64::try_from(x).is_ok() && i64::try_from(y).is_ok() {
                        //     safe_arithmetic!(op, x, y, col("x").cast(DataType::Int64), col("y").cast(DataType::Int64));
                        //     safe_arithmetic!(op, x, y, x.to_lit().cast(DataType::Int64), y.to_lit().cast(DataType::Int64));
                        // }
                        ///////////////////////////////////////////////////////////////////////////////
                    }
                }
            }
        };
    }

    #[test]
    fn i128_can_be_properly_converted_to_lit() {
        test_expr! {1_i128.to_lit(), batch!("" => [1_i128])};
        test_expr! {0_i128.to_lit(), batch!("" => [0_i128])};
        test_expr! {(-1_i128).to_lit(), batch!("" => [-1_i128])};
        test_expr! {MAX_DECIMAL.to_lit(), batch!("" => [MAX_DECIMAL])};
        test_expr! {(-MAX_DECIMAL).to_lit(), batch!("" => [-MAX_DECIMAL])};
        test_expr! {(-MAX_DECIMAL + 1).to_lit(), batch!("" => [-MAX_DECIMAL + 1])};
        test_expr! {(MAX_DECIMAL - 1).to_lit(), batch!("" => [MAX_DECIMAL - 1])};
        test_expr!(
            (i64::MAX as i128).to_lit(),
            batch!("" => [i64::MAX as i128])
        );
        test_expr!(
            (i64::MIN as i128).to_lit(),
            batch!("" => [i64::MIN as i128])
        );

        (-3000..3000).for_each(|i| {
            test_expr! {i.to_lit(), batch!("" => [i])};
        });
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_min_overflows() {
        test_expr! {i128::MIN.to_lit(), batch!("" => [i128::MIN])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_max_overflows() {
        test_expr! {i128::MAX.to_lit(), batch!("" => [i128::MAX])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_lit_with_i128_bigger_than_max_decimal_overflows() {
        test_expr! {(MAX_DECIMAL + 1).to_lit(), batch!("" => [(MAX_DECIMAL + 1)])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_smaller_than_min_decimal_overflows() {
        test_expr! {(MIN_DECIMAL - 1).to_lit(), batch!("" => [(MIN_DECIMAL - 1)])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_bigger_than_max_decimal_overflows() {
        test_expr! {(MAX_DECIMAL + 1).to_lit(), batch!("" => [(MAX_DECIMAL + 1)])};
    }

    #[test]
    #[should_panic]
    fn add_two_i128_literals_overflowing_will_panic() {
        test_expr!(
            MAX_DECIMAL.to_lit() + (1).to_lit(),
            batch!("" => [MAX_DECIMAL + 1])
        );
    }

    #[test]
    #[should_panic]
    fn add_literal_i128_and_column_overflowing_will_panic() {
        test_expr!(
            MAX_DECIMAL.to_lit() + col("x"),
            batch!("" => [MAX_DECIMAL + 1]),
            batch!("x" => [1_i128])
        );
    }

    #[test]
    #[should_panic]
    fn add_two_i128_and_columns_overflowing_will_panic() {
        test_expr!(
            col("y") + col("x"),
            batch!("" => [MAX_DECIMAL + 1]),
            batch!("x" => [1_i128], "y" => [MAX_DECIMAL])
        );
    }

    #[test]
    fn sub_two_i128_literals_can_overflow_but_may_not_panic() {
        test_expr!(
            MIN_DECIMAL.to_lit() - (MIN_DECIMAL / 10).to_lit(),
            batch!("" => [MIN_DECIMAL - (MIN_DECIMAL/10)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_two_i128_literals_overflows() {
        test_expr!(
            10_i128.to_lit() * (10_i128.pow(37)).to_lit(),
            batch!("" => [10_i128.pow(38)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_i128_column_and_literal_overflows() {
        test_expr!(
            col("x") * 10_i128.to_lit(),
            batch!("" => [10_i128.pow(38)]),
            batch!("x" => [10_i128.pow(37)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_i128_literal_and_column_overflows() {
        test_expr!(
            10_i128.to_lit() * col("x"),
            batch!("" => [10_i128.pow(38)]),
            batch!("x" => [10_i128.pow(37)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_two_i128_columns_overflows() {
        test_expr!(
            col("x") * col("y"),
            batch!("" => [10_i128.pow(38)]),
            batch!("x" => [10_i128.pow(37)], "y" => [10_i128])
        );
    }

    #[test]
    fn we_can_execute_multiple_arithmetic_operations_with_expressions() {
        batch_execute_test!([
            [0, -10],
            [MAX_DECIMAL, -1],
            [MIN_DECIMAL, 1],
            [MAX_DECIMAL, MIN_DECIMAL],
            [i64::MAX as i128, i64::MAX as i128],
            [i64::MIN as i128, i64::MIN as i128],
            [i64::MIN as i128, i64::MAX as i128],
            [-4654825170126467706_i128, 4654825170126467706_i128],
        ]);
    }

    #[test]
    fn we_can_execute_multiple_random_arithmetic_operations_with_expressions() {
        const NUM_RANDOM_VALUES: usize = 1000;
        let mut rng = rand::thread_rng();

        let rand_samples: Vec<_> = (0..NUM_RANDOM_VALUES)
            .flat_map(|_| {
                let lit1d = rng.sample(Uniform::new(MIN_DECIMAL, MAX_DECIMAL + 1));
                let lit2d = rng.sample(Uniform::new(MIN_DECIMAL, MAX_DECIMAL + 1));

                let lit1i = rng.sample(Uniform::new(MIN_I64, MAX_I64 + 1));
                let lit2i = rng.sample(Uniform::new(MIN_I64, MAX_I64 + 1));

                [[lit1i, lit2i], [lit1d, lit2d]]
            })
            .collect();

        batch_execute_test!(rand_samples);
    }

    #[test]
    #[should_panic]
    fn valid_i128_with_i64_sub_will_incorrectly_overflow() {
        let v = -4654825170126467706_i64;
        test_expr!(
            col("y") - col("x").cast(DataType::Int64),
            batch!("" => [0_i128]),
            batch!("y" => [v as i128], "x" => [v as i128])
        );
    }
}
