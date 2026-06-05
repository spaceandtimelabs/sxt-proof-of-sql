use super::test_utility::*;
use crate::{
    base::database::{owned_table_utility::*, TableRef, TableTestAccessor},
    sql::proof::{exercise_verification, VerifiableQueryResult},
};
use blitzar::proof::InnerProductProof;

#[test]
fn we_can_create_and_prove_an_empty_exec_plan() {
    let plan = empty_exec();

    // Verify metadata and references
    assert_eq!(plan.get_column_result_fields(), Vec::new());
    assert!(plan.get_column_references().is_empty());
    assert!(plan.get_table_references().is_empty());

    // We can use an empty table accessor for testing since EmptyExec doesn't reference any tables
    let accessor = TableTestAccessor::<InnerProductProof>::new_empty();

    let verifiable_res =
        VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();

    exercise_verification(&verifiable_res, &plan, &accessor, &TableRef::new("", ""));

    let res = verifiable_res
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    // EmptyExec first_round_evaluate and final_round_evaluate return a Table with 1 row and 0 columns
    let expected = owned_table(std::iter::empty());
    assert_eq!(res.num_rows(), 1);
    assert_eq!(res, expected);
}
