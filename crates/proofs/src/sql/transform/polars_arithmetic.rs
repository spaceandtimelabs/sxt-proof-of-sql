use polars::{
    error::ErrString,
    prelude::{DataType, Expr, GetOutput, PolarsError, PolarsResult, Series},
};

fn series_to_i64_slice(series: &Series) -> &[i64] {
    series
        .i64()
        .unwrap()
        .cont_slice()
        .expect("slice cannot contain nulls")
}

fn series_to_i128_slice(series: &Series) -> &[i128] {
    series
        .decimal()
        .unwrap()
        .cont_slice()
        .expect("slice cannot contain nulls")
}

fn has_zero_in_series(series: &Series) -> bool {
    match series.dtype().clone() {
        DataType::Decimal(Some(_), Some(_)) => series_to_i128_slice(series).iter().any(|&v| v == 0),
        DataType::Int64 => series_to_i64_slice(series).iter().any(|&v| v == 0),
        _ => false,
    }
}

fn will_div_overflow(num: &Series, den: &Series) -> bool {
    match (num.dtype(), den.dtype()) {
        (DataType::Int64, DataType::Int64) => {
            let num = series_to_i64_slice(num);
            let den = series_to_i64_slice(den);

            num.iter()
                .zip(den.iter())
                .any(|(n, d)| *n == i64::MIN && *d == -1)
        }
        _ => false,
    }
}

fn checked_div(series: &mut [Series]) -> PolarsResult<Option<Series>> {
    let [num, den] = [&series[0], &series[1]];

    if has_zero_in_series(den) {
        return Err(PolarsError::InvalidOperation(ErrString::from(
            "division by zero is not allowed",
        )));
    }

    if will_div_overflow(num, den) {
        return Err(PolarsError::InvalidOperation(ErrString::from(
            "attempt to divide i64 with overflow",
        )));
    }

    Ok(Some(num / den))
}

/// TODO: add docs
pub trait SafeDivision {
    /// TODO: add docs
    fn checked_div(self, rhs: Expr) -> Expr;
}

impl SafeDivision for Expr {
    fn checked_div(self, rhs: Expr) -> Expr {
        self.map_many(checked_div, &[rhs], GetOutput::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        record_batch as batch,
        sql::transform::{polars_conversions::LiteralConversion, test_utility::select, ResultExpr},
    };
    use polars::prelude::col;
    use rand::{distributions::Uniform, Rng};

    const MAX_I64: i128 = i64::MAX as i128;
    const MIN_I64: i128 = i64::MIN as i128;
    const MAX_DECIMAL: i128 = 10_i128.pow(38) - 1;
    const MIN_DECIMAL: i128 = -(10_i128.pow(38) - 1);

    macro_rules! test_expr {
        ($expr:expr, $expected:expr) => {
            let data = batch!("" => [0_i64]);
            let result = ResultExpr::new(select(&[$expr.alias("res")])).transform_results(data).unwrap();
            assert_eq!(result, $expected);
        };
        ($expr:expr, $expected:expr, $data:expr) => {
            assert_eq!(ResultExpr::new(select(&[$expr.alias("res")])).transform_results($data).unwrap(), $expected);
        };
    }

    macro_rules! safe_arithmetic {
        ($op:expr, $x:expr, $y:expr, $x_e:expr, $y_e:expr) => {
            let data = batch!("x" => [$x], "y" => [$y]);

            match $op {
                "add" => {
                    if $x.checked_add($y).is_some() && ($x + $y) <= MAX_DECIMAL && ($x + $y) >= MIN_DECIMAL {
                        test_expr!($x_e + $y_e, batch!("res" => [$x + $y]), data);
                    }
                }
                "sub" => {
                    if $x.checked_sub($y).is_some() && ($x - $y) <= MAX_DECIMAL && ($x - $y) >= MIN_DECIMAL {
                        test_expr!($x.to_lit() - $y.to_lit(), batch!("res" => [$x - $y]), data);
                    }
                }
                "mul" => {
                    if $x.checked_mul($y).is_some() && ($x * $y) <= MAX_DECIMAL && ($x * $y) >= MIN_DECIMAL {
                        test_expr!($x.to_lit() * $y.to_lit(), batch!("res" => [$x * $y]), data);
                    }
                }
                "div" => {
                    if $y != 0 {
                        test_expr!($x.to_lit().checked_div($y.to_lit()), batch!("res" => [$x / $y]), data);
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
    #[should_panic]
    fn conversion_to_literal_with_i128_min_overflows() {
        test_expr! {i128::MIN.to_lit(), batch!("res" => [i128::MIN])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_max_overflows() {
        test_expr! {i128::MAX.to_lit(), batch!("res" => [i128::MAX])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_lit_with_i128_bigger_than_max_decimal_overflows() {
        test_expr! {(MAX_DECIMAL + 1).to_lit(), batch!("res" => [(MAX_DECIMAL + 1)])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_smaller_than_min_decimal_overflows() {
        test_expr! {(MIN_DECIMAL - 1).to_lit(), batch!("res" => [(MIN_DECIMAL - 1)])};
    }

    #[test]
    #[should_panic]
    fn conversion_to_literal_with_i128_bigger_than_max_decimal_overflows() {
        test_expr! {(MAX_DECIMAL + 1).to_lit(), batch!("res" => [(MAX_DECIMAL + 1)])};
    }

    #[test]
    #[should_panic]
    fn add_two_i128_literals_overflowing_will_panic() {
        test_expr!(
            MAX_DECIMAL.to_lit() + (1_i128).to_lit(),
            batch!("res" => [MAX_DECIMAL + 1])
        );
    }

    #[test]
    #[should_panic]
    fn add_literal_i128_and_column_overflowing_will_panic() {
        test_expr!(
            MAX_DECIMAL.to_lit() + col("x"),
            batch!("res" => [MAX_DECIMAL + 1]),
            batch!("x" => [1_i128])
        );
    }

    #[test]
    #[should_panic]
    fn add_two_i128_and_columns_overflowing_will_panic() {
        test_expr!(
            col("y") + col("x"),
            batch!("res" => [MAX_DECIMAL + 1]),
            batch!("x" => [1_i128], "y" => [MAX_DECIMAL])
        );
    }

    #[test]
    fn sub_two_i128_literals_can_overflow_but_may_not_panic() {
        test_expr!(
            MIN_DECIMAL.to_lit() - (MIN_DECIMAL / 10).to_lit(),
            batch!("res" => [MIN_DECIMAL - (MIN_DECIMAL/10)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_two_i128_literals_overflows() {
        test_expr!(
            10_i128.to_lit() * (10_i128.pow(37)).to_lit(),
            batch!("res" => [MAX_DECIMAL + 1])
        );
    }

    #[test]
    #[should_panic]
    fn mul_i128_column_and_literal_overflows() {
        test_expr!(
            col("x") * 10_i128.to_lit(),
            batch!("res" => [MAX_DECIMAL + 1]),
            batch!("x" => [10_i128.pow(37)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_i128_literal_and_column_overflows() {
        test_expr!(
            10_i128.to_lit() * col("x"),
            batch!("res" => [MAX_DECIMAL + 1]),
            batch!("x" => [10_i128.pow(37)])
        );
    }

    #[test]
    #[should_panic]
    fn mul_two_i128_columns_overflows() {
        test_expr!(
            col("x") * col("y"),
            batch!("res" => [MAX_DECIMAL + 1]),
            batch!("x" => [10_i128.pow(37)], "y" => [10_i128])
        );
    }

    #[test]
    fn we_can_execute_multiple_arithmetic_operations_between_expressions() {
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
    fn we_can_execute_multiple_random_arithmetic_operations_between_expressions() {
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
            batch!("res" => [0_i128]),
            batch!("y" => [v as i128], "x" => [v as i128])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_zero_i64_numerator_zero_i64_denominator_will_error() {
        test_expr!(
            col("i1").checked_div(col("i")),
            batch!("res" => [0_i64]),
            batch!("i1" => [0_i64], "i" => [0_i64])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_non_zero_i64_numerator_zero_i64_denominator_will_error() {
        test_expr!(
            col("i1").checked_div(col("i")),
            batch!("res" => [0_i64]),
            batch!("i1" => [1_i64], "i" => [0_i64])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_non_zero_i128_numerator_zero_i128_denominator_will_error() {
        test_expr!(
            col("d1").checked_div(col("d")),
            batch!("res" => [0_i128]),
            batch!("d1" => [1_i128], "d" => [0_i128])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_zero_i128_numerator_zero_i128_denominator_will_error() {
        test_expr!(
            col("d1").checked_div(col("d")),
            batch!("res" => [0_i128]),
            batch!("d1" => [0_i128], "d" => [0_i128])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_non_zero_i64_numerator_zero_i128_denominator_will_error() {
        test_expr!(
            col("i").checked_div(col("d")),
            batch!("res" => [0_i128]),
            batch!("i" => [1_i64], "d" => [0_i128])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_zero_i64_numerator_zero_i128_denominator_will_error() {
        test_expr!(
            col("i").checked_div(col("d")),
            batch!("res" => [0_i128]),
            batch!("i" => [0_i64], "d" => [0_i128])
        );
    }

    #[test]
    #[should_panic]
    fn division_with_non_zero_i128_numerator_zero_i64_denominator_will_error() {
        test_expr!(
            col("d").checked_div(col("i")),
            batch!("res" => [0_i128]),
            batch!("i" => [0_i64], "d" => [1_i128])
        );
    }

    #[test]
    #[should_panic]
    fn polars_will_panic_with_i64_numerator_and_denominator_and_division_overflowing_even_in_release_mode(
    ) {
        test_expr!(
            col("i1").checked_div(col("i2")),
            batch!("res" => [MIN_I64 as i64]),
            batch!("i1" => [MIN_I64 as i64],
            "i2" => [-1_i64])
        );
    }

    #[test]
    fn division_with_different_values_of_numerator_and_denominator_is_valid() {
        let range = (-31..31).chain([
            MAX_I64,
            MAX_I64,
            MAX_DECIMAL,
            MIN_DECIMAL,
            MAX_I64 - 1,
            MIN_I64 + 1,
            MAX_DECIMAL - 1,
            MIN_DECIMAL + 1,
            MAX_I64 / 10,
            MIN_I64 / 10,
            MAX_DECIMAL / 10,
            MIN_DECIMAL / 10,
        ]);

        for num in range.clone() {
            for den in range.clone() {
                if den != 0 {
                    if (MIN_I64..=MAX_I64).contains(&num) && (MIN_I64..=MAX_I64).contains(&den) {
                        let (div_res, will_overflow) = (num as i64).overflowing_div(den as i64);

                        if !will_overflow {
                            test_expr!(
                                col("num").checked_div(col("den")),
                                batch!("res" => [div_res]),
                                batch!("num" => [num as i64],
                                "den" => [den as i64])
                            );
                        }
                    }

                    if (MIN_I64..=MAX_I64).contains(&num) {
                        test_expr!(
                            col("num")
                                .cast(DataType::Decimal(Some(38), Some(0)))
                                .checked_div(col("den")),
                            batch!("res" => [num / den]),
                            batch!("num" => [num as i64],
                            "den" => [den])
                        );
                    }

                    if (MIN_I64..=MAX_I64).contains(&den) {
                        test_expr!(
                            col("num")
                                .checked_div(col("den").cast(DataType::Decimal(Some(38), Some(0)))),
                            batch!("res" => [num / den]),
                            batch!("num" => [num],
                            "den" => [den as i64])
                        );
                    }

                    test_expr!(
                        col("num").checked_div(col("den")),
                        batch!("res" => [num / den]),
                        batch!("num" => [num],
                        "den" => [den])
                    );
                }
            }
        }
    }

    #[test]
    fn we_can_use_compound_arithmetic_expressions() {
        let range = (-31..31).chain([
            MIN_I64,
            MAX_I64,
            MAX_I64 - 1,
            MIN_I64 + 1,
            MAX_I64,
            MIN_I64,
            MAX_DECIMAL / 1000,
            MIN_DECIMAL / 1000,
        ]);

        for v1 in range.clone() {
            for v2 in range.clone() {
                let expr = 5_i64.to_lit()
                    * ((2_i64.to_lit() + col("v1") * 3_i64.to_lit() - col("v1"))
                        .checked_div(col("v2") + (-2_i64).to_lit() * col("v2")))
                    + 77_i64.to_lit();

                let num = 2_i128 + v1 * 3 - v1;
                let den = v2 - 2 * v2;

                if den != 0 {
                    test_expr!(
                        expr,
                        batch!("res" => [5 * (num / den) + 77]),
                        batch!("v1" => [v1], "v2" => [v2])
                    );
                }
            }
        }
    }
}
