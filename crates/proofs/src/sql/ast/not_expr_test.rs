use super::{EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, TableExpr};
use crate::base::database::{
    make_random_test_accessor, ColumnRef, ColumnType, RandomTestAccessorDescriptor, TableRef,
    TestAccessor,
};
use crate::base::scalar::ToScalar;
use crate::sql::proof::QueryExpr;
use crate::sql::proof::{exercise_verification, VerifiableQueryResult};
use proofs_sql::Identifier;

use arrow::array::Int64Array;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use indexmap::IndexMap;
use polars::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use std::sync::Arc;

#[test]
fn we_can_prove_a_not_equals_query_with_a_single_selected_row() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(NotExpr::new(Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(1u64),
        )))),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![123, 456]),
            ("b".to_string(), vec![0, 1]),
        ]),
        0_usize,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![123];
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

fn test_random_tables_with_given_offset(offset_generators: usize) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["a", "b"];
    for _ in 0..10 {
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let accessor =
            make_random_test_accessor(&mut rng, table_ref, &cols, &descr, offset_generators);
        let val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let expr = FilterExpr::new(
            vec![FilterResultExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("a").unwrap(),
                    ColumnType::BigInt,
                ),
                Identifier::try_new("a").unwrap(),
            )],
            TableExpr { table_ref },
            Box::new(NotExpr::new(Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                val.to_scalar(),
            )))),
        );
        let proof_res = VerifiableQueryResult::new(&expr, &accessor);
        exercise_verification(&proof_res, &expr, &accessor, table_ref);
        let res = proof_res.verify(&expr, &accessor).unwrap().unwrap();
        let expected = accessor.query_table(table_ref, |df| {
            df.clone()
                .lazy()
                .filter(col("b").neq(val))
                .select([col("a")])
                .collect()
                .unwrap()
        });
        assert_eq!(res, expected);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(75);
}
