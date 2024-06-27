#![cfg(feature = "test")]
use ark_std::test_rng;
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{DoryEvaluationProof, DoryProverPublicSetup},
    sql::{parse::QueryExpr, proof::QueryProof},
};

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_query_containing_extrema_with_dory() {
    use proof_of_sql::base::time::{timestamp::PoSQLTimeUnit, timezone::PoSQLTimeZone};

    let dory_prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let dory_verifier_setup = (&dory_prover_setup).into();
    let mut accessor = OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(
        dory_prover_setup.clone(),
    );
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([
            smallint("smallint", [i16::MIN, 0, i16::MAX]),
            int("int", [i32::MIN, 0, i32::MAX]),
            bigint("bigint", [i64::MIN, 0, i64::MAX]),
            int128("int128", [i128::MIN, 0, i128::MAX]),
            timestamptz(
                "times",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::UTC,
                [-2208988800, 0, 1577836800],
            ), // 1900, 1970 and 2020
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT times FROM table WHERE times = timestamp '1970-01-01T00:00:00Z'"
            .parse()
            .unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<DoryEvaluationProof>::new(query.proof_expr(), &accessor, &dory_prover_setup);
    let owned_table_result = proof
        .verify(
            query.proof_expr(),
            &accessor,
            &serialized_result,
            &dory_verifier_setup,
        )
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz(
        "times",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::UTC,
        [0],
    )]);
    assert_eq!(owned_table_result, expected_result);
}
