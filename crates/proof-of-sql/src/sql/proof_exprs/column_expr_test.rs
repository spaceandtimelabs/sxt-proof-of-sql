use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, ColumnField, ColumnType, OwnedTableTestAccessor, TableRef,
        },
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::test_utility::*,
        proof_plans::test_utility::*,
    },
};

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([boolean("a", [true, false])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        cols_expr_plan(&t, &["a"], &accessor),
        table_exec(
            t.clone(),
            vec![ColumnField::new("a".into(), ColumnType::Boolean)],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([boolean("a", [true, false])]);
    assert_eq!(res, expected_res);
}
