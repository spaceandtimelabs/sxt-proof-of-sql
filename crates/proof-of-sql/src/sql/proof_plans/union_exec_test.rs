use super::test_utility::*;
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnType, OwnedTable,
            OwnedTableTestAccessor, TableTestAccessor, TestAccessor,
        },
        map::indexmap,
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::{
            exercise_verification, FirstRoundBuilder, ProvableQueryResult, ProverEvaluate,
            VerifiableQueryResult,
        },
        proof_exprs::test_utility::*,
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_union_exec() {
    let data0 = owned_table([bigint("a0", [0_i64; 0])]);
    let t0 = "sxt.t0".parse().unwrap();
    let data1 = owned_table([bigint("a1", [0_i64; 0])]);
    let t1 = "sxt.t1".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0, data0, 0);
    accessor.add_table(t1, data1, 0);
    let ast = union_exec(
        vec![
            projection(cols_expr_plan(t1, &["a1"], &accessor), tab(t1)),
            projection(cols_expr_plan(t0, &["a0"], &accessor), tab(t0)),
        ],
        vec![column_field("a", ColumnType::BigInt)],
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t0);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [0_i64; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_union_exec() {
    let alloc = Bump::new();
    let data0 = table([
        borrowed_bigint("a0", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar("b0", ["1", "2", "3", "4", "5"], &alloc),
    ]);
    let t0 = "sxt.t0".parse().unwrap();
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = "sxt.t1".parse().unwrap();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0, data0, 0);
    accessor.add_table(t1, data1, 0);
    let ast = union_exec(
        vec![
            projection(cols_expr_plan(t0, &["a0", "b0"], &accessor), tab(t0)),
            table_exec(
                t1,
                vec![
                    column_field("a1", ColumnType::BigInt),
                    column_field("b1", ColumnType::VarChar),
                ],
            ),
        ],
        vec![
            column_field("a", ColumnType::BigInt),
            column_field("b", ColumnType::VarChar),
        ],
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t0);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5, 2, 3, 4, 5, 6]),
        varchar("b", ["1", "2", "3", "4", "5", "2", "3", "4", "5", "6"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_more_complex_union_exec() {
    let alloc = Bump::new();
    let data0 = table([
        borrowed_bigint("a0", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar("b0", ["1", "2", "3", "4", "5"], &alloc),
    ]);
    let t0 = "sxt.t0".parse().unwrap();
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = "sxt.t1".parse().unwrap();
    let data2 = table([
        borrowed_bigint("a2", [3_i64, 4, 5, 6, 7], &alloc),
        borrowed_varchar("b2", ["3", "4", "5", "6", "7"], &alloc),
    ]);
    let t2 = "sxt.t2".parse().unwrap();
    let data3 = table([
        borrowed_bigint("a3", [4_i64, 5, 6, 7, 8], &alloc),
        borrowed_varchar("b3", ["4", "5", "6", "7", "8"], &alloc),
    ]);
    let t3 = "sxt.t3".parse().unwrap();
    let data4 = table([
        borrowed_bigint("a4", [0_i64; 0], &alloc),
        borrowed_varchar("b4", [""; 0], &alloc),
    ]);
    let t4 = "sxt.t4".parse().unwrap();
    let data5 = table([
        borrowed_bigint("a5", [5_i64, 6, 7, 8], &alloc),
        borrowed_varchar("b5", ["5", "6", "7", "8"], &alloc),
    ]);
    let t5 = "sxt.t5".parse().unwrap();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0, data0, 0);
    accessor.add_table(t1, data1, 0);
    accessor.add_table(t2, data2, 0);
    accessor.add_table(t3, data3, 0);
    accessor.add_table(t4, data4, 0);
    accessor.add_table(t5, data5, 0);
    let ast = slice_exec(
        union_exec(
            vec![
                projection(cols_expr_plan(t0, &["a0", "b0"], &accessor), tab(t0)),
                projection(cols_expr_plan(t1, &["a1", "b1"], &accessor), tab(t1)),
                slice_exec(
                    filter(
                        cols_expr_plan(t2, &["a2", "b2"], &accessor),
                        tab(t2),
                        gte(column(t2, "a2", &accessor), const_smallint(5_i16)),
                    ),
                    2,
                    None,
                ),
                filter(
                    vec![
                        aliased_plan(const_bigint(105_i64), "const"),
                        col_expr_plan(t3, "b3", &accessor),
                    ],
                    tab(t3),
                    equal(column(t3, "a3", &accessor), const_int128(6_i128)),
                ),
                projection(cols_expr_plan(t4, &["a4", "b4"], &accessor), tab(t4)),
                table_exec(
                    t5,
                    vec![
                        column_field("a5", ColumnType::BigInt),
                        column_field("b5", ColumnType::VarChar),
                    ],
                ),
            ],
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::VarChar),
            ],
        ),
        4,
        Some(11),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t0);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [5_i64, 2, 3, 4, 5, 6, 7, 105, 5, 6, 7]),
        varchar("b", ["5", "2", "3", "4", "5", "6", "7", "6", "5", "6", "7"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_get_result_from_union_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data0 = table([
        borrowed_bigint("a0", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar("b0", ["1", "2", "3", "4", "5"], &alloc),
    ]);
    let t0 = "sxt.t0".parse().unwrap();
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = "sxt.t1".parse().unwrap();
    let table_map = indexmap! {
        t0 => data0.clone(),
        t1 => data1.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0, data0, 0);
    accessor.add_table(t1, data1, 0);
    let fields = vec![
        column_field("a", ColumnType::BigInt),
        column_field("b", ColumnType::VarChar),
    ];
    let ast = union_exec(
        vec![
            projection(cols_expr_plan(t0, &["a0", "b0"], &accessor), tab(t0)),
            projection(cols_expr_plan(t1, &["a1", "b1"], &accessor), tab(t1)),
        ],
        fields.clone(),
    );
    let first_round_builder = &mut FirstRoundBuilder::new();
    let res: OwnedTable<Curve25519Scalar> = ProvableQueryResult::from(ast.first_round_evaluate(
        first_round_builder,
        &alloc,
        &table_map,
    ))
    .to_owned_table(&fields)
    .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5, 2, 3, 4, 5, 6]),
        varchar("b", ["1", "2", "3", "4", "5", "2", "3", "4", "5", "6"]),
    ]);

    assert_eq!(res, expected);
}
