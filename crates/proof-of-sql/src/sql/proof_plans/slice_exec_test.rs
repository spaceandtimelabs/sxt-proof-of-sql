use super::{test_utility::*, DynProofPlan};
use crate::{
    base::{
        database::{
            owned_table_utility::*, ColumnField, ColumnType, OwnedTable, OwnedTableTestAccessor,
            TestAccessor,
        },
        map::IndexMap,
        math::decimal::Precision,
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::{
            exercise_verification, ProvableQueryResult, ProverEvaluate, ResultBuilder,
            VerifiableQueryResult,
        },
        proof_exprs::{test_utility::*, DynProofExpr},
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_slice_exec() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["1", "2", "3", "4", "5"]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = slice_exec(
        projection(
            cols_expr_plan(t, &["a", "b"], &accessor),
            tab(t),
        ),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [2_i64, 3]),
        varchar("b", ["2", "3"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_get_an_empty_result_from_a_slice_on_an_empty_table_using_result_evaluate() {
    let data = owned_table([
        bigint("a", [0; 0]),
        bigint("b", [0; 0]),
        int128("c", [0; 0]),
        varchar("d", [""; 0]),
        scalar("e", [0; 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr<RistrettoPoint> =
        equal(column(t, "a", &accessor), const_int128(999));
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        Some(2),
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(0);
    let result_cols = expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let res = ProvableQueryResult::new(&builder.result_index_vector, &result_cols)
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
    let data = owned_table([
        bigint("a", [1, 4, 5, 2, 5]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr: DynProofPlan<RistrettoPoint> = slice_exec(
        projection(cols_expr_plan(t, &["b", "c", "d", "e"], &accessor), tab(t)),
        5,
        Some(1),
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    let result_cols = expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res = ProvableQueryResult::new(&builder.result_index_vector, &result_cols)
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
    let data = owned_table([
        bigint("a", [1, 4, 5, 2, 5]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr<RistrettoPoint> =
        equal(column(t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(cols_expr_plan(t, &[], &accessor), tab(t), where_clause),
        1,
        None,
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    let result_cols = expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[];
    let res = ProvableQueryResult::new(&builder.result_index_vector, &result_cols)
        .to_owned_table::<Curve25519Scalar>(fields)
        .unwrap();
    let expected = OwnedTable::try_new(IndexMap::default()).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_the_correct_result_from_a_slice_using_result_evaluate() {
    let data = owned_table([
        bigint("a", [1, 4, 5, 2, 5]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let where_clause: DynProofExpr<RistrettoPoint> =
        equal(column(t, "a", &accessor), const_int128(5));
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        None,
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    let result_cols = expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res = ProvableQueryResult::new(&builder.result_index_vector, &result_cols)
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
fn we_can_prove_a_slice_on_an_empty_table() {
    let data = owned_table([
        bigint("a", [101; 0]),
        bigint("b", [3; 0]),
        int128("c", [3; 0]),
        varchar("d", ["3"; 0]),
        scalar("e", [3; 0]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = slice_exec(
        filter(
            cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
            tab(t),
            equal(column(t, "a", &accessor), const_int128(106)),
        ),
        1,
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("b", [3; 0]),
        int128("c", [3; 0]),
        varchar("d", ["3"; 0]),
        scalar("e", [3; 0]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_slice_with_empty_results() {
    let data = owned_table([
        bigint("a", [1_i64, 4_i64, 5_i64, 2_i64, 5_i64]),
        bigint("b", [1_i64, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let where_clause: DynProofExpr<RistrettoPoint> =
        gte(column(t, "a", &accessor), const_int128(0));
    let ast = slice_exec(
        filter(
            cols_expr_plan(t, &["b"], &accessor),
            tab(t),
            where_clause,
        ),
        1,
        Some(2),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [2_i64, 3])]);
    assert_eq!(res, expected_res);
    // let data = owned_table([
    //     bigint("a", [101, 104, 105, 102, 105]),
    //     bigint("b", [1, 2, 3, 4, 5]),
    //     //int128("c", [1, 2, 3, 4, 5]),
    //     //varchar("d", ["1", "2", "3", "4", "5"]),
    //     //scalar("e", [1, 2, 3, 4, 5]),
    // ]);
    // let t = "sxt.t".parse().unwrap();
    // let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    // accessor.add_table(t, data, 0);
    // let expr = slice_exec(
    //     projection(cols_expr_plan(t, &["b"], &accessor), tab(t)),
    //     // filter(
    //     //     cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
    //     //     tab(t),
    //     //     equal(column(t, "a", &accessor), const_int128(106)),
    //     // ),
    //     1,
    //     None,
    // );
    // let res = VerifiableQueryResult::new(&expr, &accessor, &());
    // exercise_verification(&res, &expr, &accessor, t);
    // let res = res.verify(&expr, &accessor, &()).unwrap().table;
    // let expected = owned_table([
    //     bigint("b", [104, 105, 102, 105]),
    //     //int128("c", [2]),
    //     //varchar("d", ["2"]),
    //     //scalar("e", [2]),
    // ]);
    // assert_eq!(res, expected);
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
        None,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("b", [4, 7]),
        int128("c", [4, 5]),
        varchar("d", ["4", "5"]),
        scalar("e", [4, 5]),
        int128("const", [105, 105]),
        boolean("bool", [true, false]),
    ]);
    assert_eq!(res, expected);
}
