use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, ColumnType, OwnedTableTestAccessor, SchemaAccessor, TableRef,
        },
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, DynProofExpr, ProofExpr},
        proof_plans::test_utility::*,
        AnalyzeError,
    },
};

/// A small typed accessor shared by the `try_new` validation tests.
fn schema_accessor() -> impl SchemaAccessor {
    let data = owned_table([
        boolean("flag", [true, false]),
        bigint("a", [1_i64, 2]),
        bigint("b", [3_i64, 4]),
        varchar("name", ["x", "y"]),
    ]);
    OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "t"),
        data,
        0,
        (),
    )
}

/// `try_new` accepts a well-formed CASE and reports its result type as the branch type.
#[test]
fn we_can_construct_a_valid_case_expr() {
    let t = TableRef::new("sxt", "t");
    let accessor = schema_accessor();
    let expr = case_when(
        vec![(column(&t, "flag", &accessor), column(&t, "a", &accessor))],
        column(&t, "b", &accessor),
    );
    assert_eq!(expr.data_type(), ColumnType::BigInt);
}

#[test]
fn we_cannot_construct_a_case_expr_with_non_boolean_condition() {
    let t = TableRef::new("sxt", "t");
    let accessor = schema_accessor();
    // condition is a BigInt column, not boolean
    let result = DynProofExpr::try_new_case(
        vec![(column(&t, "a", &accessor), column(&t, "a", &accessor))],
        column(&t, "b", &accessor),
    );
    assert!(matches!(result, Err(AnalyzeError::DataTypeMismatch { .. })));
}

#[test]
fn we_cannot_construct_a_case_expr_with_mismatched_branch_types() {
    let t = TableRef::new("sxt", "t");
    let accessor = schema_accessor();
    // then is BigInt, else is VarChar
    let result = DynProofExpr::try_new_case(
        vec![(column(&t, "flag", &accessor), column(&t, "a", &accessor))],
        column(&t, "name", &accessor),
    );
    assert!(matches!(result, Err(AnalyzeError::DataTypeMismatch { .. })));
}

#[test]
fn we_can_construct_a_case_expr_with_no_arms() {
    // A CASE with no WHEN arms is valid and evaluates to the else branch; forbidding
    // it (if desired) is the planner's job, not the plan's.
    let t = TableRef::new("sxt", "t");
    let accessor = schema_accessor();
    let expr = DynProofExpr::try_new_case(vec![], column(&t, "a", &accessor)).unwrap();
    assert_eq!(expr.data_type(), ColumnType::BigInt);
}

/// A single-arm numeric CASE proves and verifies, and `exercise_verification`
/// confirms tampering with the result or the proof is rejected.
#[test]
fn we_can_prove_a_single_arm_case() {
    let data = owned_table([
        boolean("flag", [true, false, true, false]),
        bigint("a", [10_i64, 20, 30, 40]),
        bigint("b", [-1_i64, -2, -3, -4]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![aliased_plan(
            case_when(
                vec![(column(&t, "flag", &accessor), column(&t, "a", &accessor))],
                column(&t, "b", &accessor),
            ),
            "res",
        )],
        table_exec(
            t.clone(),
            vec![
                column_field("flag", ColumnType::Boolean),
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    // flag picks a where true, else b
    assert_eq!(res, owned_table([bigint("res", [10_i64, -2, 30, -4])]));
}

/// A multi-arm CASE exercises the exclusive-guard commitments and first-match-wins.
#[test]
fn we_can_prove_a_multi_arm_case_with_first_match_wins() {
    let data = owned_table([
        boolean("c1", [true, false, true, false]),
        boolean("c2", [false, true, true, false]),
        bigint("a", [1_i64, 2, 3, 4]),
        bigint("b", [10_i64, 20, 30, 40]),
        bigint("z", [100_i64, 200, 300, 400]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    // CASE WHEN c1 THEN a WHEN c2 THEN b ELSE z END
    let ast = projection(
        vec![aliased_plan(
            case_when(
                vec![
                    (column(&t, "c1", &accessor), column(&t, "a", &accessor)),
                    (column(&t, "c2", &accessor), column(&t, "b", &accessor)),
                ],
                column(&t, "z", &accessor),
            ),
            "res",
        )],
        table_exec(
            t.clone(),
            vec![
                column_field("c1", ColumnType::Boolean),
                column_field("c2", ColumnType::Boolean),
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
                column_field("z", ColumnType::BigInt),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    // row 0: c1 -> a=1; row 1: c2 -> b=20; row 2: both true, first wins -> a=3;
    // row 3: neither -> z=400
    assert_eq!(res, owned_table([bigint("res", [1_i64, 20, 3, 400])]));
}

/// Varchar branches prove and verify: the prover selects real strings.
#[test]
fn we_can_prove_a_varchar_case() {
    let data = owned_table([
        boolean("flag", [true, false, true]),
        varchar("a", ["gold", "gold", "gold"]),
        varchar("b", ["silver", "silver", "silver"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![aliased_plan(
            case_when(
                vec![(column(&t, "flag", &accessor), column(&t, "a", &accessor))],
                column(&t, "b", &accessor),
            ),
            "res",
        )],
        table_exec(
            t.clone(),
            vec![
                column_field("flag", ColumnType::Boolean),
                column_field("a", ColumnType::VarChar),
                column_field("b", ColumnType::VarChar),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    assert_eq!(
        res,
        owned_table([varchar("res", ["gold", "silver", "gold"])])
    );
}

/// A CASE with no WHEN arms proves and verifies, and evaluates to the else branch.
#[test]
fn we_can_prove_a_case_with_no_arms() {
    let data = owned_table([bigint("a", [10_i64, 20, 30])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![aliased_plan(
            case_when(vec![], column(&t, "a", &accessor)),
            "res",
        )],
        table_exec(t.clone(), vec![column_field("a", ColumnType::BigInt)]),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    assert_eq!(res, owned_table([bigint("res", [10_i64, 20, 30])]));
}
