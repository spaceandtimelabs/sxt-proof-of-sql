use super::test_utility::*;
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnType, OwnedTable,
            OwnedTableTestAccessor, TableRef, TableTestAccessor, TestAccessor,
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
fn we_can_prove_and_get_the_correct_empty_result_from_a_union_with_no_tables() {
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let ast = union_exec(
        vec![],
        vec![
            column_field("a", ColumnType::BigInt),
            column_field("b", ColumnType::VarChar),
        ],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [0_i64; 0]), varchar("b", [""; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_union_with_one_table() {
    let data = owned_table([
        bigint("a0", [0_i64, 1, 2, 3, 4]),
        varchar("b0", ["", "1", "2", "3", "4"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let ast = union_exec(
        vec![filter(
            cols_expr_plan(&t, &["a0"], &accessor),
            tab(&t),
            gte(column(&t, "a0", &accessor), const_int128(2_i128)),
        )],
        vec![column_field("a", ColumnType::BigInt)],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2_i64, 3, 4])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_union_exec() {
    let data0 = owned_table([bigint("a0", [0_i64; 0])]);
    let t0 = TableRef::new("sxt", "t0");
    let data1 = owned_table([bigint("a1", [0_i64; 0])]);
    let t1 = TableRef::new("sxt", "t1");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);
    let ast = union_exec(
        vec![
            projection(
                cols_expr_plan(&t1, &["a1"], &accessor),
                table_exec(t1, vec![column_field("a1", ColumnType::BigInt)]),
            ),
            projection(
                cols_expr_plan(&t0, &["a0"], &accessor),
                table_exec(t0.clone(), vec![column_field("a0", ColumnType::BigInt)]),
            ),
        ],
        vec![column_field("a", ColumnType::BigInt)],
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &t0);
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
    let t0 = TableRef::new("sxt", "t0");
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = TableRef::new("sxt", "t1");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);
    let ast = union_exec(
        vec![
            projection(
                cols_expr_plan(&t0, &["a0", "b0"], &accessor),
                table_exec(
                    t0.clone(),
                    vec![
                        column_field("a0", ColumnType::BigInt),
                        column_field("b0", ColumnType::VarChar),
                    ],
                ),
            ),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &t0);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5, 2, 3, 4, 5, 6]),
        varchar("b", ["1", "2", "3", "4", "5", "2", "3", "4", "5", "6"]),
    ]);
    assert_eq!(res, expected_res);
}

#[allow(clippy::too_many_lines)]
#[test]
fn we_can_prove_and_get_the_correct_result_from_a_more_complex_union_exec() {
    let alloc = Bump::new();
    let data0 = table([
        borrowed_bigint("a0", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar("b0", ["1", "2", "3", "4", "5"], &alloc),
    ]);
    let t0 = TableRef::new("sxt", "t0");
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = TableRef::new("sxt", "t1");
    let data2 = table([
        borrowed_bigint("a2", [3_i64, 4, 5, 6, 7], &alloc),
        borrowed_varchar("b2", ["3", "4", "5", "6", "7"], &alloc),
    ]);
    let t2 = TableRef::new("sxt", "t2");
    let data3 = table([
        borrowed_bigint("a3", [4_i64, 5, 6, 7, 8], &alloc),
        borrowed_varchar("b3", ["4", "5", "6", "7", "8"], &alloc),
    ]);
    let t3 = TableRef::new("sxt", "t3");
    let data4 = table([
        borrowed_bigint("a4", [0_i64; 0], &alloc),
        borrowed_varchar("b4", [""; 0], &alloc),
    ]);
    let t4 = TableRef::new("sxt", "t4");
    let data5 = table([
        borrowed_bigint("a5", [5_i64, 6, 7, 8], &alloc),
        borrowed_varchar("b5", ["5", "6", "7", "8"], &alloc),
    ]);
    let t5 = TableRef::new("sxt", "t5");
    let data6 = table([
        borrowed_bigint("a6", [7_i64, 8, 9, 10], &alloc),
        borrowed_varchar("b6", ["8", "9", "10", "11"], &alloc),
    ]);
    let t6 = TableRef::new("sxt", "t6");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);
    accessor.add_table(t2.clone(), data2, 0);
    accessor.add_table(t3.clone(), data3, 0);
    accessor.add_table(t4.clone(), data4, 0);
    accessor.add_table(t5.clone(), data5, 0);
    accessor.add_table(t6.clone(), data6, 0);
    let ast = union_exec(
        vec![
            slice_exec(
                union_exec(
                    vec![
                        projection(
                            cols_expr_plan(&t0, &["a0", "b0"], &accessor),
                            table_exec(
                                t0.clone(),
                                vec![
                                    column_field("a0", ColumnType::BigInt),
                                    column_field("b0", ColumnType::VarChar),
                                ],
                            ),
                        ),
                        projection(
                            cols_expr_plan(&t1, &["a1", "b1"], &accessor),
                            table_exec(
                                t1.clone(),
                                vec![
                                    column_field("a1", ColumnType::BigInt),
                                    column_field("b1", ColumnType::VarChar),
                                ],
                            ),
                        ),
                        slice_exec(
                            filter(
                                cols_expr_plan(&t2, &["a2", "b2"], &accessor),
                                tab(&t2),
                                gte(column(&t2, "a2", &accessor), const_smallint(5_i16)),
                            ),
                            2,
                            None,
                        ),
                        filter(
                            vec![
                                aliased_plan(const_bigint(105_i64), "const"),
                                col_expr_plan(&t3, "b3", &accessor),
                            ],
                            tab(&t3),
                            equal(column(&t3, "a3", &accessor), const_int128(6_i128)),
                        ),
                        projection(
                            cols_expr_plan(&t4, &["a4", "b4"], &accessor),
                            table_exec(
                                t4.clone(),
                                vec![
                                    column_field("a4", ColumnType::BigInt),
                                    column_field("b4", ColumnType::VarChar),
                                ],
                            ),
                        ),
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
            ),
            table_exec(
                t6,
                vec![
                    column_field("a6", ColumnType::BigInt),
                    column_field("b6", ColumnType::VarChar),
                ],
            ),
        ],
        vec![
            column_field("a", ColumnType::BigInt),
            column_field("b", ColumnType::VarChar),
        ],
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &t0);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [5_i64, 2, 3, 4, 5, 6, 7, 105, 5, 6, 7, 7, 8, 9, 10]),
        varchar(
            "b",
            [
                "5", "2", "3", "4", "5", "6", "7", "6", "5", "6", "7", "8", "9", "10", "11",
            ],
        ),
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
    let len_0 = data0.num_rows();
    let t0 = TableRef::new("sxt", "t0");
    let data1 = table([
        borrowed_bigint("a1", [2_i64, 3, 4, 5, 6], &alloc),
        borrowed_varchar("b1", ["2", "3", "4", "5", "6"], &alloc),
    ]);
    let t1 = TableRef::new("sxt", "t1");

    let table_map = indexmap! {
        t0.clone() => data0.clone(),
        t1.clone() => data1.clone()
    };

    let len_1 = data1.num_rows();

    let data_length = std::cmp::max(len_0, len_1);

    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);
    let fields = vec![
        column_field("a", ColumnType::BigInt),
        column_field("b", ColumnType::VarChar),
    ];
    let ast = union_exec(
        vec![
            projection(
                cols_expr_plan(&t0, &["a0", "b0"], &accessor),
                table_exec(
                    t0.clone(),
                    vec![
                        column_field("a0", ColumnType::BigInt),
                        column_field("b0", ColumnType::VarChar),
                    ],
                ),
            ),
            projection(
                cols_expr_plan(&t1, &["a1", "b1"], &accessor),
                table_exec(
                    t1.clone(),
                    vec![
                        column_field("a1", ColumnType::BigInt),
                        column_field("b1", ColumnType::VarChar),
                    ],
                ),
            ),
        ],
        fields.clone(),
    );
    let first_round_builder = &mut FirstRoundBuilder::new(data_length);
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
