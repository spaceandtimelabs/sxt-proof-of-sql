use super::{
    test_utility::{aliased_plan, cast, column, tab},
    LiteralExpr,
};
use crate::{
    base::{
        database::{
            owned_table_utility::{
                bigint, boolean, decimal75, int, int128, owned_table, smallint, timestamptz,
                tinyint,
            },
            ColumnType, LiteralValue, OwnedTableTestAccessor, TableRef,
        },
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::DynProofExpr,
        proof_plans::test_utility::filter,
        AnalyzeError,
    },
};
use blitzar::proof::InnerProductProof;

#[test]
fn we_can_prove_a_simple_cast_expr() {
    let data = owned_table([
        boolean("a", [false, true, false, true]),
        boolean("b", [true, true, false, true]),
        boolean("c", [false, false, false, true]),
        boolean("d", [false, true, false, false]),
        boolean("e", [false, true, true, false]),
        timestamptz(
            "f",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::new(1),
            [1i64, -500, i64::MAX, 0],
        ),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            aliased_plan(
                cast(column(&t, "a", &accessor), ColumnType::TinyInt),
                "a_cast",
            ),
            aliased_plan(
                cast(column(&t, "b", &accessor), ColumnType::SmallInt),
                "b_cast",
            ),
            aliased_plan(cast(column(&t, "c", &accessor), ColumnType::Int), "c_cast"),
            aliased_plan(
                cast(column(&t, "d", &accessor), ColumnType::BigInt),
                "d_cast",
            ),
            aliased_plan(
                cast(column(&t, "e", &accessor), ColumnType::Int128),
                "e_cast",
            ),
            aliased_plan(
                cast(column(&t, "f", &accessor), ColumnType::BigInt),
                "f_cast",
            ),
        ],
        tab(&t),
        super::DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        tinyint("a_cast", [0i8, 1, 0, 1]),
        smallint("b_cast", [1i16, 1, 0, 1]),
        int("c_cast", [0i32, 0, 0, 1]),
        bigint("d_cast", [0i64, 1, 0, 0]),
        int128("e_cast", [0i128, 1, 1, 0]),
        bigint("f_cast", [1i64, -500, i64::MAX, 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_get_error_if_we_cast_uncastable_type() {
    let data = owned_table([decimal75("a", 57, 2, [1_i16, 2, 3, 4])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    assert!(matches!(
        DynProofExpr::try_new_cast(column(&t, "a", &accessor), ColumnType::BigInt),
        Err(AnalyzeError::DataTypeMismatch { .. })
    ));
}
