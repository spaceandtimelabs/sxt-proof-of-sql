use super::{test_utility::*, DynProofPlan, ProjectionExec};
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnRef, ColumnType,
            OwnedTable, OwnedTableTestAccessor, TableRef, TableTestAccessor, TestAccessor,
        },
        map::{indexmap, IndexMap, IndexSet},
        math::decimal::Precision,
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::{
            exercise_verification, FirstRoundBuilder, ProofPlan, ProvableQueryResult,
            ProverEvaluate, VerifiableQueryResult,
        },
        proof_exprs::{test_utility::*, ColumnExpr, DynProofExpr},
        proof_plans::TableExec,
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use proof_of_sql_parser::{Identifier, ResourceId};

#[test]
fn we_can_correctly_fetch_the_query_result_schema() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let a = Identifier::try_new("a").unwrap();
    let b = Identifier::try_new("b").unwrap();
    let provable_ast = ProjectionExec::new(
        vec![
            aliased_plan(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref,
                    a,
                    ColumnType::BigInt,
                ))),
                "a",
            ),
            aliased_plan(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref,
                    b,
                    ColumnType::BigInt,
                ))),
                "b",
            ),
        ],
        Box::new(DynProofPlan::Table(TableExec::new(
            table_ref,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
            ],
        ))),
    );
    let column_fields: Vec<ColumnField> = provable_ast.get_column_result_fields();
    assert_eq!(
        column_fields,
        vec![
            ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
            ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ]
    );
}

#[test]
fn we_can_correctly_fetch_all_the_referenced_columns() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let a = Identifier::try_new("a").unwrap();
    let f = Identifier::try_new("f").unwrap();
    let provable_ast = ProjectionExec::new(
        vec![
            aliased_plan(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref,
                    a,
                    ColumnType::BigInt,
                ))),
                "a",
            ),
            aliased_plan(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref,
                    f,
                    ColumnType::BigInt,
                ))),
                "f",
            ),
        ],
        Box::new(DynProofPlan::Table(TableExec::new(
            table_ref,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("f".parse().unwrap(), ColumnType::BigInt),
            ],
        ))),
    );

    let ref_columns = provable_ast.get_column_references();

    assert_eq!(
        ref_columns,
        IndexSet::from_iter([
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt
            ),
            ColumnRef::new(
                table_ref,
                Identifier::try_new("f").unwrap(),
                ColumnType::BigInt
            ),
        ])
    );

    let ref_tables = provable_ast.get_table_references();

    assert_eq!(ref_tables, IndexSet::from_iter([table_ref]));
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_basic_projection() {
    let data = owned_table([
        bigint("a", [1_i64, 4_i64, 5_i64, 2_i64, 5_i64, 1, 4, 5, 2, 5]),
        bigint("b", [1_i64, 2, 3, 4, 5, 1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let ast = projection(
        cols_expr_plan(t, &["b"], &accessor),
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected = owned_table([bigint("b", [1_i64, 2, 3, 4, 5, 1, 2, 3, 4, 5])]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_nontrivial_projection() {
    let data = owned_table([
        bigint("a", [1_i64, 4_i64, 5_i64, 2_i64, 5_i64]),
        bigint("b", [1_i64, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let ast = projection(
        vec![
            aliased_plan(add(column(t, "b", &accessor), const_bigint(1)), "b"),
            aliased_plan(
                multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
                "prod",
            ),
        ],
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("b", [2_i64, 3, 4, 5, 6]),
        bigint("prod", [1_i64, 8, 15, 8, 25]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_composed_projection() {
    let data = owned_table([
        bigint("a", [1_i64, 4_i64, 5_i64, 2_i64, 5_i64]),
        bigint("b", [1_i64, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let ast = projection(
        vec![
            aliased_plan(add(column(t, "b", &accessor), const_bigint(1)), "b"),
            aliased_plan(
                multiply(column(t, "a", &accessor), column(t, "b", &accessor)),
                "prod",
            ),
        ],
        filter(
            vec![
                aliased_plan(add(column(t, "b", &accessor), const_bigint(1)), "b"),
                aliased_plan(
                    add(column(t, "a", &accessor), column(t, "b", &accessor)),
                    "a",
                ),
            ],
            tab(t),
            equal(column(t, "a", &accessor), const_int128(5)),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected = owned_table([bigint("b", [5_i64, 7]), bigint("prod", [32_i64, 60])]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_an_empty_result_from_a_basic_projection_on_an_empty_table_using_result_evaluate() {
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
    let expr: DynProofPlan = projection(
        cols_expr_plan(t, &["b", "c", "d", "e"], &accessor),
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
                ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
                ColumnField::new("e".parse().unwrap(), ColumnType::Scalar),
            ],
        ),
    );

    let mut builder = FirstRoundBuilder::new();
    expr.first_round_evaluate(&mut builder);
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
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map))
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
fn we_can_get_no_columns_from_a_basic_projection_with_no_selected_columns_using_result_evaluate() {
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
    let expr: DynProofPlan = projection(
        cols_expr_plan(t, &[], &accessor),
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
                ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
                ColumnField::new("e".parse().unwrap(), ColumnType::Scalar),
            ],
        ),
    );
    let mut builder = FirstRoundBuilder::new();
    expr.first_round_evaluate(&mut builder);
    let fields = &[];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map))
            .to_owned_table(fields)
            .unwrap();
    let expected = OwnedTable::try_new(IndexMap::default()).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_the_correct_result_from_a_basic_projection_using_result_evaluate() {
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
    let expr: DynProofPlan = projection(
        vec![
            aliased_plan(add(column(t, "b", &accessor), const_bigint(1)), "b"),
            aliased_plan(
                multiply(column(t, "b", &accessor), column(t, "c", &accessor)),
                "prod",
            ),
            col_expr_plan(t, "d", &accessor),
            aliased_plan(const_decimal75(1, 0, 3), "e"),
        ],
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
                ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
                ColumnField::new("e".parse().unwrap(), ColumnType::Scalar),
            ],
        ),
    );
    let mut builder = FirstRoundBuilder::new();
    expr.first_round_evaluate(&mut builder);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("prod".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res: OwnedTable<Curve25519Scalar> =
        ProvableQueryResult::from(expr.result_evaluate(&alloc, &table_map))
            .to_owned_table(fields)
            .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [2, 3, 4, 5, 6]),
        int128("prod", [1, 4, 9, 16, 25]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        decimal75("e", 1, 0, [3; 5]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_projection_on_an_empty_table() {
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
    let expr = projection(
        vec![
            aliased_plan(add(column(t, "b", &accessor), const_bigint(1)), "b"),
            aliased_plan(
                multiply(column(t, "b", &accessor), column(t, "c", &accessor)),
                "prod",
            ),
            col_expr_plan(t, "d", &accessor),
            aliased_plan(const_decimal75(1, 0, 3), "e"),
        ],
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
                ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
                ColumnField::new("e".parse().unwrap(), ColumnType::Scalar),
            ],
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("b", [3; 0]),
        int128("prod", [3; 0]),
        varchar("d", ["3"; 0]),
        decimal75("e", 1, 0, [3; 0]),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_prove_a_projection() {
    let data = owned_table([
        bigint("a", [101, 104, 105, 102, 105]),
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t, data, 0);
    let expr = projection(
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
        table_exec(
            t,
            vec![
                ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
                ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
                ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
                ColumnField::new("e".parse().unwrap(), ColumnType::Scalar),
            ],
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &());
    exercise_verification(&res, &expr, &accessor, t);
    let res = res.verify(&expr, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("b", [1, 2, 3, 4, 7]),
        int128("c", [1, 3, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
        int128("const", [105; 5]),
        boolean("bool", [true, false, true, true, false]),
    ]);
    assert_eq!(res, expected);
}
