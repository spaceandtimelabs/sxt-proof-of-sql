use super::test_utility::*;
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnType, OwnedTable,
            OwnedTableTestAccessor, TableRef, TableTestAccessor, TestAccessor,
        },
        map::{indexmap, IndexMap},
        math::decimal::Precision,
        proof::ProofError,
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::{
        proof::{
            exercise_verification, FirstRoundBuilder, ProvableQueryResult, ProverEvaluate,
            QueryError, VerifiableQueryResult,
        },
        proof_exprs::{test_utility::*, DynProofExpr},
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_slice_exec() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["1", "2", "3", "4", "5"]),
    ]);
    let t: TableRef = "sxt.t".parse().unwrap();
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = slice_exec(
        projection(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            table_exec(
                t.clone(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::VarChar),
                ],
            ),
        ),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([bigint("a", [2_i64, 3]), varchar("b", ["2", "3"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_slice_exec() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["1", "2", "3", "4", "5"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let where_clause: DynProofExpr = equal(column(&t, "a", &accessor), const_int128(2));
    let ast = slice_exec(
        filter(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            tab(&t),
            where_clause,
        ),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([bigint("a", [0_i64; 0]), varchar("b", [""; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_get_an_empty_result_from_a_slice_on_an_empty_table_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [0; 0], &alloc),
        borrowed_bigint("b", [0; 0], &alloc),
        borrowed_int128("c", [0; 0], &alloc),
        borrowed_varchar("d", [""; 0], &alloc),
        borrowed_scalar("e", [0; 0], &alloc),
    ]);
    let data_length = data.num_rows();
    let t = TableRef::new("sxt", "t");
    let table_map = indexmap! {
        t.clone() => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let where_clause: DynProofExpr = equal(column(&t, "a", &accessor), const_int128(999));
    let expr = slice_exec(
        filter(
            cols_expr_plan(&t, &["b", "c", "d", "e"], &accessor),
            tab(&t),
            where_clause,
        ),
        1,
        Some(2),
    );

    let fields = &[
        ColumnField::new("b".into(), ColumnType::BigInt),
        ColumnField::new("c".into(), ColumnType::Int128),
        ColumnField::new("d".into(), ColumnType::VarChar),
        ColumnField::new(
            "e".into(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let first_round_builder = &mut FirstRoundBuilder::new(data_length);
    let res: OwnedTable<Curve25519Scalar> = ProvableQueryResult::from(
        expr.first_round_evaluate(first_round_builder, &alloc, &table_map, &[])
            .unwrap(),
    )
    .to_owned_table(fields)
    .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [0; 0]),
        int128("c", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 75, 0, [0; 0]),
    ]);

    assert_eq!(res, expected);
}

#[test]
fn we_can_get_an_empty_result_from_a_slice_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let data_length = data.num_rows();
    let t = TableRef::new("sxt", "t");
    let table_map = indexmap! {
        t.clone() => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let where_clause: DynProofExpr = equal(column(&t, "a", &accessor), const_int128(999));
    let expr = slice_exec(
        filter(
            cols_expr_plan(&t, &["b", "c", "d", "e"], &accessor),
            tab(&t),
            where_clause,
        ),
        1,
        Some(2),
    );

    let fields = &[
        ColumnField::new("b".into(), ColumnType::BigInt),
        ColumnField::new("c".into(), ColumnType::Int128),
        ColumnField::new("d".into(), ColumnType::VarChar),
        ColumnField::new(
            "e".into(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let first_round_builder = &mut FirstRoundBuilder::new(data_length);
    let res: OwnedTable<Curve25519Scalar> = ProvableQueryResult::from(
        expr.first_round_evaluate(first_round_builder, &alloc, &table_map, &[])
            .unwrap(),
    )
    .to_owned_table(fields)
    .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [0; 0]),
        int128("c", [0; 0]),
        varchar("d", [""; 0]),
        decimal75("e", 1, 0, [0; 0]),
    ]);

    assert_eq!(res, expected);
}

#[test]
fn we_can_get_no_columns_from_a_slice_with_empty_input_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let data_length = data.num_rows();
    let t = TableRef::new("sxt", "t");
    let table_map = indexmap! {
        t.clone() => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let where_clause: DynProofExpr = equal(column(&t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(cols_expr_plan(&t, &[], &accessor), tab(&t), where_clause),
        2,
        None,
    );
    let fields = &[];
    let first_round_builder = &mut FirstRoundBuilder::new(data_length);
    let res: OwnedTable<Curve25519Scalar> = ProvableQueryResult::from(
        expr.first_round_evaluate(first_round_builder, &alloc, &table_map, &[])
            .unwrap(),
    )
    .to_owned_table(fields)
    .unwrap();
    let expected = OwnedTable::try_new(IndexMap::default()).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_the_correct_result_from_a_slice_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let data_length = data.num_rows();
    let t = TableRef::new("sxt", "t");
    let table_map = indexmap! {
        t.clone() => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let where_clause: DynProofExpr = equal(column(&t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(
            cols_expr_plan(&t, &["b", "c", "d", "e"], &accessor),
            tab(&t),
            where_clause,
        ),
        1,
        None,
    );
    let fields = &[
        ColumnField::new("b".into(), ColumnType::BigInt),
        ColumnField::new("c".into(), ColumnType::Int128),
        ColumnField::new("d".into(), ColumnType::VarChar),
        ColumnField::new(
            "e".into(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let first_round_builder = &mut FirstRoundBuilder::new(data_length);
    let res: OwnedTable<Curve25519Scalar> = ProvableQueryResult::from(
        expr.first_round_evaluate(first_round_builder, &alloc, &table_map, &[])
            .unwrap(),
    )
    .to_owned_table(fields)
    .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [5]),
        int128("c", [5]),
        varchar("d", ["5"]),
        decimal75("e", 1, 0, [5]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_slice_exec() {
    let data = owned_table([
        bigint("a", [101, 105, 105, 105, 105]),
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = slice_exec(
        filter(
            vec![
                col_expr_plan(&t, "b", &accessor),
                col_expr_plan(&t, "c", &accessor),
                col_expr_plan(&t, "d", &accessor),
                col_expr_plan(&t, "e", &accessor),
                aliased_plan(const_int128(105), "const"),
                aliased_plan(
                    equal(column(&t, "b", &accessor), column(&t, "c", &accessor)),
                    "bool",
                ),
            ],
            tab(&t),
            equal(column(&t, "a", &accessor), const_int128(105)),
        ),
        2,
        Some(1),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("b", [4]),
        int128("c", [4]),
        varchar("d", ["4"]),
        scalar("e", [4]),
        int128("const", [105]),
        boolean("bool", [true]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_nested_slice_exec() {
    let data = owned_table([
        bigint("a", [101, 105, 105, 105, 105]),
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(&t, "b", &accessor),
                    col_expr_plan(&t, "c", &accessor),
                    col_expr_plan(&t, "d", &accessor),
                    col_expr_plan(&t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(&t, "b", &accessor), column(&t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(&t),
                equal(column(&t, "a", &accessor), const_int128(105)),
            ),
            1,
            Some(3),
        ),
        1,
        Some(1),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("b", [4]),
        int128("c", [4]),
        varchar("d", ["4"]),
        scalar("e", [4]),
        int128("const", [105]),
        boolean("bool", [true]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_nested_slice_exec_with_no_rows() {
    let data = owned_table([
        bigint("a", [101, 105, 105, 105, 105]),
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(&t, "b", &accessor),
                    col_expr_plan(&t, "c", &accessor),
                    col_expr_plan(&t, "d", &accessor),
                    col_expr_plan(&t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(&t, "b", &accessor), column(&t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(&t),
                equal(column(&t, "a", &accessor), const_int128(105)),
            ),
            1,
            Some(3),
        ),
        3,
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("b", [0; 0]),
        int128("c", [0; 0]),
        varchar("d", [""; 0]),
        scalar("e", [0; 0]),
        int128("const", [0; 0]),
        boolean("bool", [true; 0]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_another_nested_slice_exec_with_no_rows() {
    let data = owned_table([
        bigint("a", [101, 105, 105, 105, 105]),
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(&t, "b", &accessor),
                    col_expr_plan(&t, "c", &accessor),
                    col_expr_plan(&t, "d", &accessor),
                    col_expr_plan(&t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(&t, "b", &accessor), column(&t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(&t),
                equal(column(&t, "a", &accessor), const_int128(105)),
            ),
            6,
            Some(3),
        ),
        3,
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("b", [0; 0]),
        int128("c", [0; 0]),
        varchar("d", [""; 0]),
        scalar("e", [0; 0]),
        int128("const", [0; 0]),
        boolean("bool", [true; 0]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_create_and_prove_a_slice_exec_on_top_of_a_table_exec() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("namespace", "table_name");
    let plan = slice_exec(
        table_exec(
            table_ref.clone(),
            vec![
                ColumnField::new("language_rank".into(), ColumnType::BigInt),
                ColumnField::new("language_name".into(), ColumnType::VarChar),
                ColumnField::new("space_and_time".into(), ColumnType::VarChar),
            ],
        ),
        1,
        Some(4),
    );
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref.clone(),
        table([
            borrowed_bigint("language_rank", [0_i64, 1, 2, 3], &alloc),
            borrowed_varchar(
                "language_name",
                ["English", "Español", "Português", "Français"],
                &alloc,
            ),
            borrowed_varchar(
                "space_and_time",
                [
                    "space and time",
                    "espacio y tiempo",
                    "espaço e tempo",
                    "espace et temps",
                ],
                &alloc,
            ),
        ]),
        0_usize,
        (),
    );
    let verifiable_res = VerifiableQueryResult::new(&plan, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &plan, &accessor, &table_ref);
    let res = verifiable_res
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected = owned_table([
        bigint("language_rank", [1_i64, 2, 3]),
        varchar("language_name", ["Español", "Português", "Français"]),
        varchar(
            "space_and_time",
            ["espacio y tiempo", "espaço e tempo", "espace et temps"],
        ),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_create_and_prove_a_slice_exec_on_top_of_an_empty_exec() {
    let empty_table = owned_table([]);
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let expr = slice_exec(empty_exec(), 3, Some(2));
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &(), &[]).unwrap();
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    assert_eq!(res, empty_table);
}

#[test]
fn we_cannot_prove_a_slice_exec_if_it_has_groupby_as_input_for_now() {
    let data = owned_table([
        bigint("a", [1, 2, 2, 1, 2]),
        bigint("b", [99, 99, 99, 99, 0]),
        bigint("c", [101, 102, 103, 104, 105]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = slice_exec(
        group_by(
            cols_expr(&t, &["a"], &accessor),
            vec![sum_expr(column(&t, "c", &accessor), "sum_c")],
            "__count__",
            tab(&t),
            equal(column(&t, "b", &accessor), const_int128(99)),
        ),
        2,
        None,
    );
    let res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    assert!(matches!(
        res.verify(&expr, &accessor, &(), &[]),
        Err(QueryError::ProofError {
            source: ProofError::UnsupportedQueryPlan { .. }
        })
    ));
}
