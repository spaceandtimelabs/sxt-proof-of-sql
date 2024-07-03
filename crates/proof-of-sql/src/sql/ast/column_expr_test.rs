use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, OwnedTableTestAccessor},
    },
    sql::{
        ast::test_utility::*,
        proof::{exercise_verification, VerifiableQueryResult},
    },
};

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([boolean("a", [true, false])]);
    let t = "sxt.t".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
    let ast = projection(cols_expr_plan(t, &["a"], &accessor), tab(t));
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, t);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([boolean("a", [true, false])]);
    assert_eq!(res, expected_res);
}
