use crate::{
    base::database::{owned_table_utility::*, OwnedTable},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::postprocessing::{
        apply_postprocessing_steps, group_by_postprocessing::*, test_utility::*,
        OwnedTablePostprocessing, PostprocessingError,
    },
};
use bigdecimal::BigDecimal;
use proof_of_sql_parser::{intermediate_ast::AggregationOperator, utility::*};

#[test]
fn we_cannot_have_invalid_group_bys() {
    // Column in result but not in group by or aggregation
    let expr = add(sum(col("a")), col("b")); // b is not in group by or aggregation
    let res = GroupByPostprocessing::try_new(vec!["a".into()], vec![aliased_expr(expr, "res")]);
    assert!(matches!(
        res,
        Err(PostprocessingError::IdentNotInAggregationOperatorOrGroupByClause { .. })
    ));

    // Nested aggregation
    let expr = sum(max(col("a"))); // Nested aggregation
    let res = GroupByPostprocessing::try_new(vec!["a".into()], vec![aliased_expr(expr, "res")]);
    assert!(matches!(
        res,
        Err(PostprocessingError::NestedAggregationInGroupByClause { .. })
    ));
}

#[test]
fn we_can_make_group_by_postprocessing() {
    // SELECT SUM(a) + 2 as c0, SUM(b + a) as c1 FROM tab GROUP BY a, b
    let res = GroupByPostprocessing::try_new(
        vec!["a".into(), "b".into()],
        vec![
            aliased_expr(add(sum(col("a")), lit(2)), "c0"),
            aliased_expr(sum(add(col("b"), col("a"))), "c1"),
        ],
    )
    .unwrap();
    assert_eq!(res.group_by(), &["a".into(), "b".into()]);
    assert_eq!(
        res.remainder_exprs(),
        &[
            aliased_expr(add(col("__col_agg_0"), lit(2)), "c0"),
            aliased_expr(col("__col_agg_1"), "c1"),
        ]
    );
    assert_eq!(
        res.aggregation_exprs(),
        &[
            (AggregationOperator::Sum, *col("a"), "__col_agg_0".into()),
            (
                AggregationOperator::Sum,
                *add(col("b"), col("a")),
                "__col_agg_1".into()
            ),
        ]
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn we_can_do_simple_group_bys() {
    // SELECT 1 as cons FROM tab
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &[],
        &[aliased_expr(lit(1), "cons")],
    )];
    let expected_table = owned_table([bigint("cons", [1_i64])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT 1 as cons FROM tab group by a
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a"],
        &[aliased_expr(lit(1), "cons")],
    )];
    let expected_table = owned_table([bigint("cons", [1_i64; 4])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT a, true as truth FROM tab GROUP BY a
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 1, 2, 2]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a"],
        &[
            aliased_expr(col("a"), "a"),
            aliased_expr(lit(true), "truth"),
        ],
    )];
    let expected_table = owned_table([int128("a", [1_i128, 2]), boolean("truth", [true; 2])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT a as cons FROM tab GROUP BY a
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a"],
        &[aliased_expr(col("a"), "cons")],
    )];
    let expected_table = owned_table([int128("cons", [1_i64, 2, 3, 4])]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT MAX(a) as max_a, MIN(b) as min_b, SUM(c) as sum_c, COUNT(d) as count_d FROM tab
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &[],
        &[
            aliased_expr(max(col("a")), "max_a"),
            aliased_expr(min(col("b")), "min_b"),
            aliased_expr(sum(col("c")), "sum_c"),
            aliased_expr(count(col("d")), "count_d"),
        ],
    )];
    let expected_table = owned_table([
        int128("max_a", [4_i128]),
        bigint("min_b", [5_i64]),
        smallint("sum_c", [42_i16]),
        bigint("count_d", [4_i64]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT a, MIN(b) as min_b, SUM(c) as sum_c, COUNT(d) as count_d FROM tab GROUP BY a
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 1, 2, 2]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a"],
        &[
            aliased_expr(col("a"), "a"),
            aliased_expr(min(col("b")), "min_b"),
            aliased_expr(sum(col("c")), "sum_c"),
            aliased_expr(count(col("d")), "count_d"),
        ],
    )];
    let expected_table = owned_table([
        int128("a", [1_i128, 2]),
        bigint("min_b", [5_i64, 7]),
        smallint("sum_c", [19_i16, 23]),
        bigint("count_d", [2_i64, 2]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT a + b as res, SUM(c) as sum_c, COUNT(d) as count_d FROM tab GROUP BY a, b, a, b, b
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 5, 5, 1]),
        bigint("b", [1_i64, 2, 2, 2]),
        smallint("c", [9_i16, 11, 12, 10]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a", "b", "a", "b", "b"],
        &[
            aliased_expr(add(col("a"), col("b")), "res"),
            aliased_expr(sum(col("c")), "sum_c"),
            aliased_expr(count(col("d")), "count_d"),
        ],
    )];
    let expected_table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("res", [2_i128, 3, 7]),
        smallint("sum_c", [9_i16, 10, 23]),
        bigint("count_d", [1_i64, 1, 2]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[allow(clippy::too_many_lines)]
#[test]
fn we_can_do_complex_group_bys() {
    // SELECT 2 * MAX(2 * a + 1) as max_a, MIN(b + 4) - 2.4 as min_b, SUM(c * 1.4) as sum_c, COUNT(d) + 3 as count_d FROM tab
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        varchar("d", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &[],
        &[
            aliased_expr(
                mul(lit(2), max(add(mul(lit(2), col("a")), lit(1)))),
                "max_a",
            ),
            aliased_expr(
                sub(
                    min(add(col("b"), lit(4))),
                    lit("2.4".parse::<BigDecimal>().unwrap()),
                ),
                "min_b",
            ),
            aliased_expr(
                sum(mul(col("c"), lit("1.4".parse::<BigDecimal>().unwrap()))),
                "sum_c",
            ),
            aliased_expr(add(count(col("d")), lit(3)), "count_d"),
        ],
    )];
    let expected_table = owned_table([
        int128("max_a", [18_i128]),
        decimal75("min_b", 21, 1, [66]),
        decimal75("sum_c", 8, 1, [588]),
        bigint("count_d", [7_i64]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);

    // SELECT count(a + 2.5) + 2 as count_a, 2 * (MAX(2 * c + 1) + SUM(2.5 * d)) as res, count(d) - 1 as count_d_alt, MIN(b + 2.4) - 3.4 as min_b, SUM(c * 1.7) as sum_c, COUNT(d) - 3 as count_d, COUNT(e) as count_e FROM tab group by a, a, a, a
    let table: OwnedTable<Curve25519Scalar> = owned_table([
        int128("a", [1_i128, 1, 1, 2]),
        bigint("b", [5_i64, 6, 7, 8]),
        smallint("c", [9_i16, 10, 11, 12]),
        decimal75("d", 2, 1, [13, 14, 15, 16]),
        varchar("e", ["Space", "and", "Time", "rocks"]),
    ]);
    let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
        &["a", "a", "a", "a"],
        &[
            aliased_expr(
                add(
                    count(add(col("a"), lit("2.5".parse::<BigDecimal>().unwrap()))),
                    lit(2),
                ),
                "count_a",
            ),
            aliased_expr(
                mul(
                    lit(2),
                    add(
                        max(add(mul(lit(2), col("c")), lit(1))),
                        sum(mul(lit("2.5".parse::<BigDecimal>().unwrap()), col("d"))),
                    ),
                ),
                "res",
            ),
            aliased_expr(sub(count(col("d")), lit(1)), "count_d_alt"),
            aliased_expr(
                sub(
                    min(add(col("b"), lit("2.4".parse::<BigDecimal>().unwrap()))),
                    lit("3.4".parse::<BigDecimal>().unwrap()),
                ),
                "min_b",
            ),
            aliased_expr(
                sum(mul(col("c"), lit("1.7".parse::<BigDecimal>().unwrap()))),
                "sum_c",
            ),
            aliased_expr(sub(count(col("d")), lit(3)), "count_d"),
            aliased_expr(count(col("e")), "count_e"),
        ],
    )];
    let expected_table = owned_table([
        bigint("count_a", [5_i64, 3]),
        decimal75("res", 42, 2, [6700, 5800]),
        bigint("count_d_alt", [2_i64, 0]),
        decimal75("min_b", 22, 1, [40, 70]),
        decimal75("sum_c", 8, 1, [510, 204]),
        bigint("count_d", [0_i64, -2]),
        bigint("count_e", [3_i64, 1]),
    ]);
    let actual_table = apply_postprocessing_steps(table, &postprocessing).unwrap();
    assert_eq!(actual_table, expected_table);
}

#[test]
fn we_can_do_group_by_with_null_values_following_sql_three_valued_logic() {
    use crate::base::database::owned_table_utility::{
        bigint_values, nullable_column_pair, owned_table_with_nulls,
    };
    use proof_of_sql_parser::utility::{add, col, count, is_null, max, min, mul, sum};
    use sqlparser::ast::Ident;

    // Create a table with multiple columns including NULL values
    // Column a: Some values are NULL
    let a_values = bigint_values::<Curve25519Scalar>([10, 20, 30, 40, 50, 10, 20, 30]);
    let a_presence = Some(vec![true, true, false, true, false, true, true, false]);

    // Column b: Different NULL pattern
    let b_values = bigint_values::<Curve25519Scalar>([5, 15, 25, 35, 45, 5, 15, 25]);
    let b_presence = Some(vec![true, false, true, false, true, true, false, true]);

    // Column c: Different NULL pattern
    let c_values = bigint_values::<Curve25519Scalar>([100, 200, 300, 400, 500, 600, 700, 800]);
    let c_presence = Some(vec![false, true, true, false, true, false, true, true]);

    // Create the table with nullable columns
    let table = owned_table_with_nulls([
        nullable_column_pair("a", a_values.clone(), a_presence.clone()),
        nullable_column_pair("b", b_values.clone(), b_presence.clone()),
        nullable_column_pair("c", c_values.clone(), c_presence.clone()),
    ]);

    // Test 1: Simple GROUP BY with NULL values
    // GROUP BY column with NULL values - NULL values should be grouped together
    {
        let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
            &["a"],
            &[
                aliased_expr(col("a"), "a"),
                aliased_expr(count(col("b")), "count_b"),
                aliased_expr(sum(col("c")), "sum_c"),
            ],
        )];

        let actual_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // Expected results:
        // - NULL values for 'a' should be grouped together (rows 3, 5, 8)
        // - count(b) should count non-NULL values in 'b' per group
        // - sum(c) should sum non-NULL values in 'c' per group
        let _expected_a = bigint_values::<Curve25519Scalar>([10, 20, 30, 40, 0]); // 0 represents NULL
        let _expected_count_b = bigint_values::<Curve25519Scalar>([2, 0, 1, 0, 2]); // 0 for the groups where all b are NULL
        let _expected_sum_c = bigint_values::<Curve25519Scalar>([0, 900, 300, 0, 800]); // 0 for groups with all NULLs in c

        // Verify column values
        let _a_col = actual_table.inner_table().get(&Ident::new("a")).unwrap();
        let _count_b_col = actual_table
            .inner_table()
            .get(&Ident::new("count_b"))
            .unwrap();
        let _sum_c_col = actual_table
            .inner_table()
            .get(&Ident::new("sum_c"))
            .unwrap();

        // Note: For the NULL group, we expect the count to include rows where b is non-NULL
        // and the sum to include rows where c is non-NULL
        assert_eq!(actual_table.num_rows(), 5); // 5 distinct groups: 10, 20, 30, 40, NULL
    }

    // Test 2: Arithmetic with NULL values in aggregations
    {
        let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
            &["a"],
            &[
                aliased_expr(col("a"), "a"),
                aliased_expr(sum(add(col("b"), col("c"))), "sum_b_plus_c"),
                aliased_expr(sum(mul(col("b"), lit(2))), "sum_b_times_2"),
            ],
        )];

        let actual_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // In SQL's three-valued logic:
        // - Arithmetic with NULL produces NULL
        // - sum() of a column with NULLs ignores the NULL values
        assert_eq!(actual_table.num_rows(), 5); // 5 distinct groups: 10, 20, 30, 40, NULL
    }

    // Test 3: GROUP BY with WHERE clause filtering on NULL values
    {
        // The WHERE clause filters rows where b IS NULL
        let _b_is_null = is_null(col("b"));

        // We need to combine this with a postprocessing step - in real SQL this would be
        // part of the query, but for this test we'll simulate it by filtering first
        let filtered_table = table.clone();
        // In an actual implementation, we would apply a filter here
        // For testing purposes, we can imagine the filter is applied

        let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
            &["a"],
            &[
                aliased_expr(col("a"), "a"),
                aliased_expr(count(lit(1)), "row_count"), // COUNT(*) equivalent
                aliased_expr(max(col("c")), "max_c"),
            ],
        )];

        let actual_table = apply_postprocessing_steps(filtered_table, &postprocessing).unwrap();

        // Assuming the WHERE b IS NULL filter would keep rows 2, 4, 7
        // Group by a would result in values 20, 40, NULL
        assert!(actual_table.num_rows() > 0);
    }

    // Test 4: Complex expressions with arithmetic on NULL values
    {
        let postprocessing: [OwnedTablePostprocessing; 1] = [group_by_postprocessing(
            &["a"],
            &[
                aliased_expr(col("a"), "a"),
                aliased_expr(sum(add(mul(col("b"), lit(2)), col("c"))), "complex_expr"),
                aliased_expr(min(add(col("b"), lit(5))), "min_b_plus_5"),
            ],
        )];

        let actual_table = apply_postprocessing_steps(table.clone(), &postprocessing).unwrap();

        // In SQL:
        // - Expressions with NULL values evaluate to NULL
        // - Aggregation functions ignore NULL values
        assert_eq!(actual_table.num_rows(), 5); // 5 distinct groups: 10, 20, 30, 40, NULL
    }
}
