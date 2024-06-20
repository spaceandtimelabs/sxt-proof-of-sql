use super::{test_utility::*, ProvableExpr};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            make_random_test_accessor_owned_table, owned_table_utility::*, Column, ColumnType,
            OwnedTableTestAccessor, RandomTestAccessorDescriptor, TestAccessor,
        },
    },
    sql::{
        ast::{
            test_utility::{and, equal},
            ProvableExprPlan,
        },
        proof::{exercise_verification, VerifiableQueryResult},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use itertools::{multizip, MultiUnzip};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;

#[test]
fn we_can_prove_a_simple_and_query() {
    let data = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0, 2, 2, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        and(
            equal(column(t, "b", &accessor), const_scalar(1)),
            equal(column(t, "d", &accessor), const_scalar("t")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2]), varchar("d", ["t"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_and_query_with_128_bits() {
    let data = owned_table([
        int128("a", [1, 2, 3, 4]),
        int128("b", [0, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        int128("c", [0, 2, 2, 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = dense_filter(
        cols_expr_plan(t, &["a", "d"], &accessor),
        tab(t),
        and(
            equal(column(t, "b", &accessor), const_scalar(1)),
            equal(column(t, "d", &accessor), const_scalar("t")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([int128("a", [2]), varchar("d", ["t"])]);
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
        let data = make_random_test_accessor_owned_table(&mut rng, &cols, &descr);
        let filter_val1 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let filter_val1 = format!("s{filter_val1}");
        let filter_val2 = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);

        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t,
            data.clone(),
            offset,
            (),
        );
        let ast = dense_filter(
            cols_expr_plan(t, &["a", "d"], &accessor),
            tab(t),
            and(
                equal(
                    column(t, "b", &accessor),
                    const_scalar(filter_val1.as_str()),
                ),
                equal(column(t, "c", &accessor), const_scalar(filter_val2)),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
        exercise_verification(&verifiable_res, &ast, &accessor, t);
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;

        let (expected_a, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 && c == &filter_val2 {
                Some((*a, d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("d", expected_d)]);

        assert_eq!(expected_result, res)
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

#[test]
fn we_can_compute_the_correct_output_of_an_and_expr_using_result_evaluate() {
    let data = owned_table([
        bigint("a", [1, 2, 3, 4]),
        bigint("b", [0, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0, 2, 2, 0]),
    ]);
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    let and_expr: ProvableExprPlan<RistrettoPoint> = and(
        equal(column(t, "b", &accessor), const_int128(1)),
        equal(column(t, "d", &accessor), const_varchar("t")),
    );
    let alloc = Bump::new();
    let res = and_expr.result_evaluate(4, &alloc, &accessor);
    let expected_res = Column::Boolean(&[false, true, false, false]);
    assert_eq!(res, expected_res);
}
