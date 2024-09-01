use crate::{
    base::{
        database::{
            owned_table_utility::*, ColumnField, ColumnRef, ColumnType, LiteralValue, OwnedTable,
            OwnedTableTestAccessor, TableRef, TestAccessor,
        },
        math::decimal::Precision,
        scalar::Curve25519Scalar,
    },
    proof_primitive::dory::DoryCommitment,
    sql::{
        ast::{
            test_utility::*, ColumnExpr, DynProofExpr, FilterExec, FilterResultExpr, LiteralExpr,
            TableExpr,
        },
        proof::{
            exercise_verification, ProofExecutionPlan, ProverEvaluate, ResultBuilder,
            VerifiableQueryResult,
        },
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;
use indexmap::{IndexMap, IndexSet};
use proof_of_sql_parser::{Identifier, ResourceId};

#[test]
fn we_can_correctly_fetch_the_query_result_schema() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let provable_ast = FilterExec::<RistrettoPoint>::new(
        vec![
            FilterResultExpr::new(ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            )),
            FilterResultExpr::new(ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            )),
        ],
        TableExpr { table_ref },
        DynProofExpr::try_new_equals(
            DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                table_ref,
                Identifier::try_new("c").unwrap(),
                ColumnType::BigInt,
            ))),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(123))),
        )
        .unwrap(),
    );
    let column_fields: Vec<ColumnField> = provable_ast.get_column_result_fields();
    assert_eq!(
        column_fields,
        vec![
            ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
            ColumnField::new("b".parse().unwrap(), ColumnType::BigInt)
        ]
    );
}

#[test]
fn we_can_correctly_fetch_all_the_referenced_columns() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let provable_ast = FilterExec::new(
        vec![
            FilterResultExpr::new(ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            )),
            FilterResultExpr::new(ColumnRef::new(
                table_ref,
                Identifier::try_new("f").unwrap(),
                ColumnType::BigInt,
            )),
        ],
        TableExpr { table_ref },
        not(and(
            or(
                DynProofExpr::<DoryCommitment>::try_new_equals(
                    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                        table_ref,
                        Identifier::try_new("f").unwrap(),
                        ColumnType::BigInt,
                    ))),
                    DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(45))),
                )
                .unwrap(),
                DynProofExpr::<DoryCommitment>::try_new_equals(
                    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                        table_ref,
                        Identifier::try_new("c").unwrap(),
                        ColumnType::BigInt,
                    ))),
                    DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(-2))),
                )
                .unwrap(),
            ),
            DynProofExpr::<DoryCommitment>::try_new_equals(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref,
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ))),
                DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(3))),
            )
            .unwrap(),
        )),
    );

    let ref_columns = provable_ast.get_column_references();

    assert_eq!(
        ref_columns,
        IndexSet::from([
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
            ColumnRef::new(
                table_ref,
                Identifier::try_new("c").unwrap(),
                ColumnType::BigInt
            ),
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt
            )
        ])
    );
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_basic_filter() {
    let data = owned_table([
        bigint("a", [1_i64, 4_i64, 5_i64, 2_i64, 5_i64]),
        bigint("b", [1_i64, 2, 3, 4, 5]),
    ]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let where_clause = equal(column(t, "a", &accessor), const_int128(5));
    let ast = filter(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([bigint("b", [3_i64, 5])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_get_an_empty_result_from_a_basic_filter_on_an_empty_table_using_result_evaluate() {
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
    let expr = filter(
        cols_result(t, &["b", "c", "d", "e"], &accessor),
        tab(t),
        where_clause,
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(0);
    expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let res = builder
        .make_provable_query_result()
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
fn we_can_get_an_empty_result_from_a_basic_filter_using_result_evaluate() {
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
        equal(column(t, "a", &accessor), const_int128(999));
    let expr = filter(
        cols_result(t, &["b", "c", "d", "e"], &accessor),
        tab(t),
        where_clause,
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res = builder
        .make_provable_query_result()
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
fn we_can_get_no_columns_from_a_basic_filter_with_no_selected_columns_using_result_evaluate() {
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
    let expr = filter(cols_result(t, &[], &accessor), tab(t), where_clause);
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[];
    let res = builder
        .make_provable_query_result()
        .to_owned_table::<Curve25519Scalar>(fields)
        .unwrap();
    let expected = OwnedTable::try_new(IndexMap::new()).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn we_can_get_the_correct_result_from_a_basic_filter_using_result_evaluate() {
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
    let expr = filter(
        cols_result(t, &["b", "c", "d", "e"], &accessor),
        tab(t),
        where_clause,
    );
    let alloc = Bump::new();
    let mut builder = ResultBuilder::new(5);
    expr.result_evaluate(&mut builder, &alloc, &accessor);
    let fields = &[
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("c".parse().unwrap(), ColumnType::Int128),
        ColumnField::new("d".parse().unwrap(), ColumnType::VarChar),
        ColumnField::new(
            "e".parse().unwrap(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0),
        ),
    ];
    let res = builder
        .make_provable_query_result()
        .to_owned_table(fields)
        .unwrap();
    let expected: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("b", [3, 5]),
        int128("c", [3, 5]),
        varchar("d", ["3", "5"]),
        decimal75("e", 1, 0, [3, 5]),
    ]);

    assert_eq!(res, expected);
}
