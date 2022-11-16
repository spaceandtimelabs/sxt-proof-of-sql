use super::{EqualsExpr, FilterExpr, FilterResultExpr, OrExpr, TableExpr};

use crate::base::database::{
    make_random_test_accessor, make_schema, RandomTestAccessorDescriptor, TestAccessor,
};
use crate::base::scalar::IntoScalar;
use crate::sql::proof::{exercise_verification, VerifiableQueryResult};

use arrow::array::Int64Array;
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn we_can_prove_a_simple_or_query() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new("B".to_string(), Scalar::from(1u64))),
            Box::new(EqualsExpr::new("B".to_string(), Scalar::from(2u64))),
        )),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4]),
            ("B".to_string(), vec![0, 1, 0, 2]),
        ]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![2, 4];
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_or_query_where_both_lhs_and_rhs_are_true() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new("B".to_string(), Scalar::from(1u64))),
            Box::new(EqualsExpr::new("C".to_string(), Scalar::from(2u64))),
        )),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4]),
            ("B".to_string(), vec![0, 1, 0, 1]),
            ("C".to_string(), vec![0, 2, 2, 0]),
        ]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![2, 3, 4];
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_query_random_tables() {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["A", "B", "C"];
    for _ in 0..10 {
        let accessor = make_random_test_accessor(&mut rng, "T", &cols, &descr);
        let lhs_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let rhs_val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let expr = FilterExpr::new(
            vec![FilterResultExpr::new("A".to_string())],
            TableExpr {
                name: "T".to_string(),
            },
            Box::new(OrExpr::new(
                Box::new(EqualsExpr::new("B".to_string(), lhs_val.into_scalar())),
                Box::new(EqualsExpr::new("C".to_string(), rhs_val.into_scalar())),
            )),
        );
        let res = VerifiableQueryResult::new(&expr, &accessor);
        exercise_verification(&res, &expr, &accessor);
        let res = res.verify(&expr, &accessor).unwrap().unwrap();
        let expected = accessor.query_table("T", |df| {
            df.clone()
                .lazy()
                .filter(col("B").eq(lhs_val).or(col("C").eq(rhs_val)))
                .select([col("A")])
                .collect()
                .unwrap()
        });
        assert_eq!(res, expected);
    }
}
