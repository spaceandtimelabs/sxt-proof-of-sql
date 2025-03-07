use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, OwnedTable, OwnedTableTestAccessor, TableRef},
        scalar::Curve25519Scalar,
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::test_utility::*,
        proof_plans::test_utility::*,
    },
};

#[test]
fn we_can_query_with_varbinary_equality() {
    // Create a table with bigint and varbinary columns
    let data: OwnedTable<Curve25519Scalar> = owned_table([
        bigint("a", [123, 4567]),
        varbinary("b", [&[1, 2, 3], &[4, 5, 6, 7]]),
    ]);
    
    // Create table reference and accessor
    let t = TableRef::new("sxt", "table");
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    
    // Build query plan: SELECT a, b FROM table WHERE b = [4,5,6,7]
    let ast = filter(
        cols_expr_plan(&t, &["a", "b"], &accessor),
        tab(&t),
        equal(
            column(&t, "b", &accessor), 
            const_varbinary(&[4, 5, 6, 7])
        ),
    );
    
    // Execute and verify query
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    
    // Expected result: only the second row should be returned
    let expected_res = owned_table([
        bigint("a", [4567]),
        varbinary("b", [&[4, 5, 6, 7]]),
    ]);
    
    assert_eq!(res, expected_res);
}