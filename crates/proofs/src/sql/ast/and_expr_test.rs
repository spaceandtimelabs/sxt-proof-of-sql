use crate::base::database::{
    make_random_test_accessor_data, ColumnType, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::polynomial::ArkScalar;
use crate::record_batch;
use crate::sql::ast::test_expr::TestExprNode;
use crate::sql::ast::test_utility::{and, equal};

use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

fn create_test_and_expr<
    T1: Into<ArkScalar> + Copy + Literal,
    T2: Into<ArkScalar> + Copy + Literal,
>(
    table_ref: &str,
    results: &[&str],
    lhs: (&str, T1),
    rhs: (&str, T2),
    data: RecordBatch,
    offset: usize,
) -> TestExprNode {
    let mut accessor = TestAccessor::new();
    let t = table_ref.parse().unwrap();
    accessor.add_table(t, data, offset);
    let and_expr = and(
        equal(t, lhs.0, lhs.1, &accessor),
        equal(t, rhs.0, rhs.1, &accessor),
    );
    let df_filter = polars::prelude::col(lhs.0)
        .eq(lit(lhs.1))
        .and(polars::prelude::col(rhs.0).eq(lit(rhs.1)));
    TestExprNode::new(t, results, and_expr, df_filter, accessor)
}

#[test]
fn we_can_prove_a_simple_and_query() {
    let data = record_batch!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 1, 0, 1],
        "d" => ["ab", "t", "efg", "g"],
        "c" => [0, 2, 2, 0],
    );
    let test_expr = create_test_and_expr("sxt.t", &["a", "d"], ("b", 1), ("d", "t"), data, 0);
    let res = test_expr.verify_expr();
    let expected_res = record_batch!(
        "a" => [2],
        "d" => ["t"]
    );
    assert_eq!(res, expected_res);
}

fn test_random_tables_with_given_offset(offset: usize) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = [
        ("a", ColumnType::BigInt),
        ("b", ColumnType::VarChar),
        ("c", ColumnType::BigInt),
        ("d", ColumnType::VarChar),
    ];
    for _ in 0..20 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let filter_val1 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let filter_val2 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let test_expr = create_test_and_expr(
            "sxt.t",
            &["a", "d"],
            (
                "b",
                ("s".to_owned() + &filter_val1.to_string()[..]).as_str(),
            ),
            ("c", filter_val2),
            data,
            offset,
        );
        let res = test_expr.verify_expr();
        let expected_res = test_expr.query_table();
        assert_eq!(res, expected_res);
    }
}

#[test]
fn we_can_query_random_tables_using_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_using_a_non_zero_offset() {
    test_random_tables_with_given_offset(123);
}
