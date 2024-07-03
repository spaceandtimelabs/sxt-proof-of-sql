#![cfg(feature = "test")]
#[cfg(feature = "blitzar")]
use proof_of_sql::base::commitment::InnerProductProof;
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        test_rng, DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
        PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::QueryProof},
};
use proof_of_sql_parser::intermediate_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_query_containing_rfc3339_timestamp_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
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
                [i64::MIN, 0, i64::MAX],
            ),
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
        QueryProof::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &serialized_result, &())
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

#[test]
fn we_can_prove_a_basic_query_containing_rfc3339_timestamp_with_dory() {
    let public_parameters = PublicParameters::rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
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
                [i64::MIN, 0, i64::MAX],
            ),
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
