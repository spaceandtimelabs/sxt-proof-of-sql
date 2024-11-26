use super::test_utility::*;
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnType, OwnedTable,
            OwnedTableTestAccessor, TableTestAccessor, TestAccessor,
        },
        map::{indexmap, IndexMap},
        math::decimal::Precision,
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::{
            exercise_verification, ProvableQueryResult, ProverEvaluate, VerifiableQueryResult,
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
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = slice_exec(
        projection(cols_expr_plan(t, &["a", "b"], &accessor), tab(t)),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [2_i64, 3]), varchar("b", ["2", "3"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_slice_exec() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["1", "2", "3", "4", "5"]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let where_clause: DynProofExpr = equal(column(t, "a", &accessor), const_int128(2));
    let ast = slice_exec(
        filter(
            cols_expr_plan(t, &["a", "b"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("a", [0_i64; 0]), varchar("b", [""; 0])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_get_an_empty_result_from_a_slice_on_an_empty_table_using_result_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [0; 0], &alloc),
        borrowed_bigint("b", [0; 0], &alloc),
        borrowed_int128("c", [0; 0], &alloc),
        borrowed_varchar("d", [""; 0], &alloc),
        borrowed_scalar("e", [0; 0], &alloc),
    ]);
    let t = "sxt.t".parse().unwrap();
    let table_map = indexmap! {
        t => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr = equal(column(t, "a", &accessor), const_int128(999));
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        Some(2),
    );

    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map).0)
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
fn we_can_get_an_empty_result_from_a_slice_using_result_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let t = "sxt.t".parse().unwrap();
    let table_map = indexmap! {
        t => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr = equal(column(t, "a", &accessor), const_int128(999));
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        Some(2),
    );

    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map).0)
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
fn we_can_get_no_columns_from_a_slice_with_empty_input_using_result_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let t = "sxt.t".parse().unwrap();
    let table_map = indexmap! {
        t => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr = equal(column(t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(cols_expr_plan(t, &[], &accessor), tab(t), where_clause),
        2,
        None,
    );
    let fields = &[];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map).0)
            .to_owned_table(fields)
            .unwrap();
    let expected = OwnedTable::try_new(IndexMap::default()).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_the_correct_result_from_a_slice_using_result_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [1, 2, 3, 4, 5], &alloc),
        borrowed_varchar("d", ["1", "2", "3", "4", "5"], &alloc),
        borrowed_scalar("e", [1, 2, 3, 4, 5], &alloc),
    ]);
    let t = "sxt.t".parse().unwrap();
    let table_map = indexmap! {
        t => data.clone()
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr = equal(column(t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        None,
    );
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map).0)
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
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = slice_exec(
        filter(
            vec![
                col_expr_plan(t, "b", &accessor),
                col_expr_plan(t, "c", &accessor),
                col_expr_plan(t, "d", &accessor),
                col_expr_plan(t, "e", &accessor),
                aliased_plan(const_int128(105), "const"),
                aliased_plan(
                    equal(column(t, "b", &accessor), column(t, "c", &accessor)),
                    "bool",
                ),
            ],
            tab(t),
            equal(column(t, "a", &accessor), const_int128(105)),
        ),
        2,
        Some(1),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
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
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(t, "b", &accessor),
                    col_expr_plan(t, "c", &accessor),
                    col_expr_plan(t, "d", &accessor),
                    col_expr_plan(t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(t, "b", &accessor), column(t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(t),
                equal(column(t, "a", &accessor), const_int128(105)),
            ),
            1,
            Some(3),
        ),
        1,
        Some(1),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
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
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(t, "b", &accessor),
                    col_expr_plan(t, "c", &accessor),
                    col_expr_plan(t, "d", &accessor),
                    col_expr_plan(t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(t, "b", &accessor), column(t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(t),
                equal(column(t, "a", &accessor), const_int128(105)),
            ),
            1,
            Some(3),
        ),
        3,
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
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
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = slice_exec(
        slice_exec(
            filter(
                vec![
                    col_expr_plan(t, "b", &accessor),
                    col_expr_plan(t, "c", &accessor),
                    col_expr_plan(t, "d", &accessor),
                    col_expr_plan(t, "e", &accessor),
                    aliased_plan(const_int128(105), "const"),
                    aliased_plan(
                        equal(column(t, "b", &accessor), column(t, "c", &accessor)),
                        "bool",
                    ),
                ],
                tab(t),
                equal(column(t, "a", &accessor), const_int128(105)),
            ),
            6,
            Some(3),
        ),
        3,
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
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
