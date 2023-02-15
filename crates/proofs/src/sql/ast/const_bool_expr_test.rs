use super::{ConstBoolExpr, FilterExpr, FilterResultExpr, TableExpr};
use crate::base::database::{
    make_random_test_accessor_data, ColumnRef, ColumnType, RandomTestAccessorDescriptor, TableRef,
    TestAccessor,
};
use crate::sql::proof::QueryExpr;
use crate::sql::proof::{exercise_verification, VerifiableQueryResult};
use proofs_sql::Identifier;

use arrow::array::Int64Array;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use rand::rngs::StdRng;
use rand_core::SeedableRng;
use std::sync::Arc;

fn test_random_tables_with_given_constant(value: bool) {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["a"];
    for _ in 0..10 {
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let mut accessor = TestAccessor::new();
        accessor.add_table(table_ref, data, 0_usize);

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
            Box::new(ConstBoolExpr::new(value)),
        );
        let proof_res = VerifiableQueryResult::new(&expr, &accessor);
        exercise_verification(&proof_res, &expr, &accessor, table_ref);
        let res = proof_res.verify(&expr, &accessor).unwrap().unwrap();
        let expected = accessor.query_table(table_ref, |df| {
            df.clone()
                .lazy()
                .filter(lit(value))
                .select([col("a")])
                .collect()
                .unwrap()
        });
        assert_eq!(res, expected);
    }
}

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
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
        Box::new(ConstBoolExpr::new(true)),
    );
    let data = df!("a" => [123]).unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
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

#[test]
fn we_can_prove_a_query_with_a_single_non_selected_row() {
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
        Box::new(ConstBoolExpr::new(false)),
    );
    let data = df!("a" => Vec::<i64>::new()).unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![];
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

#[test]
fn we_can_select_from_tables_with_an_always_true_where_caluse() {
    test_random_tables_with_given_constant(true);
}

#[test]
fn we_can_select_from_tables_with_an_always_false_where_clause() {
    test_random_tables_with_given_constant(false);
}
