#![cfg(feature = "test")]
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        test_rng, DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
        PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::QueryProof},
};
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_query_containing_rfc3339_timestamp_with_dory() {
    use proof_of_sql_parser::posql_time::PoSQLTimestamp;

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
                [
                    "1969-12-31T23:59:59Z", // -1
                    "1970-01-01T00:00:00Z", // 0
                    "1970-01-01T00:00:01Z", // 1
                ]
                .iter()
                .map(|s| PoSQLTimestamp::try_from(s).unwrap().timestamp.timestamp()),
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT times FROM table WHERE times = timestamp '1970-01-01T00:00:00Z';"
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
        ["1970-01-01T00:00:00Z"]
            .iter()
            .map(|s| PoSQLTimestamp::try_from(s).unwrap().timestamp.timestamp()),
    )]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_timestamp_inequality_queries_with_multiple_columns() {
    use proof_of_sql::{proof_primitive::dory::DoryCommitment, sql::utils::TimestampData};

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
            timestamptz(
                "a",
                PoSQLTimeUnit::Second,
                vec![
                    "2009-01-03T18:15:05Z", // Bitcoin genesis block time
                    "1961-04-12T06:07:00Z", // First human spaceflight by Yuri Gagarin
                    "1969-07-20T20:17:40Z", // Apollo 11 moon landing
                    "1983-01-01T00:00:00Z", // Official start of the Internet (TCP/IP)
                    "1927-03-07T00:00:00Z", // Discovery of Penicillin
                    "2004-02-04T00:00:00Z", // Founding of Facebook
                    "1964-05-20T00:00:00Z", // Cosmic Microwave Background Radiation discovered
                ]
                .to_timestamps(PoSQLTimeUnit::Second),
            ),
            timestamptz(
                "b",
                PoSQLTimeUnit::Second,
                vec![
                    "1953-02-28T00:00:00Z", // Publication of DNA's double helix structure
                    "1970-01-01T00:00:00Z", // Unix epoch
                    "1954-12-23T00:00:00Z", // First successful kidney transplant
                    "1993-04-30T00:00:00Z", // World Wide Web goes live
                    "1905-11-21T00:00:00Z", // Einstein's paper on mass-energy equivalence, E=mc²
                    "2003-04-14T00:00:00Z", // Completion of the first draft of the human genome
                    "2011-11-26T05:17:57Z", // Curiosity Rover lands on Mars
                ]
                .to_timestamps(PoSQLTimeUnit::Second),
            ),
        ]),
        0,
    );
    let query = QueryExpr::<DoryCommitment>::try_new(
        "select *, a <= b as res from TABLE where a <= b"
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
    let expected_result = owned_table([
        timestamptz(
            "a",
            PoSQLTimeUnit::Second,
            vec![
                "1961-04-12T06:07:00Z",
                "1983-01-01T00:00:00Z",
                "1964-05-20T00:00:00Z",
            ]
            .to_timestamps(PoSQLTimeUnit::Second),
        ),
        timestamptz(
            "b",
            PoSQLTimeUnit::Second,
            vec![
                "1970-01-01T00:00:00Z",
                "1993-04-30T00:00:00Z",
                "2011-11-26T05:17:57Z",
            ]
            .to_timestamps(PoSQLTimeUnit::Second),
        ),
        boolean("res", [true, true, true]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[cfg(feature = "blitzar")]
#[cfg(feature = "test")]
mod tests {

    use proof_of_sql::sql::utils::run_timestamp_query_test;
    use proof_of_sql_parser::posql_time::PoSQLTimeUnit;

    #[test]
    fn test_basic_timestamp_query() {
        let test_timestamps: Vec<i64> = vec![1609459200, 1612137600, 1614556800]; // example timestamps
        let expected_timestamps: Vec<i64> = vec![1609459200];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2021-01-01T00:00:00Z';",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_leap_seconds_parsing() {
        // Unix time for 1998-12-31T23:59:59 UTC is 915148799
        // Assuming leap second at 1998-12-31T23:59:60 UTC is recognized, it would be 915148799
        // Unix time for 1999-01-01T00:00:00 UTC is 915148800
        let test_timestamps = vec![915148799, 915148800, 915148801];

        // Test the query to select the leap second
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '1998-12-31T23:59:60Z'",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &vec![915148799],
            PoSQLTimeUnit::Second,
        );

        // Test the query to select the leap second
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '1999-01-01T00:00:00Z';",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &vec![915148800],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_new_years_eve_boundary() {
        let test_timestamps = vec!["2023-12-31T23:59:59Z", "2024-01-01T00:00:00Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-01-01T00:00:00Z';",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &vec![test_timestamps[1]],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_february_29_leap_year() {
        // Test year 2024 which is a leap year
        let test_timestamps = vec!["2024-02-29T12:00:00Z", "2024-03-01T12:00:00Z"];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-02-29T12:00:00Z';",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &vec![test_timestamps[0]],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_time_zone_crossings() {
        // Checking how the same absolute moment is handled in different time zones
        let test_timestamps = vec![
            "2023-08-15T15:00:00-05:00", // Central Time
            "2023-08-15T16:00:00-04:00", // Eastern Time, same moment
        ];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2023-08-15T20:00:00Z'", // UTC time
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &test_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_basic_unix_epoch() {
        // Parse the RFC 3339 formatted string to Unix timestamps directly
        let test_timestamps = vec![
            "2009-01-03T18:15:05Z", // The test timestamp from RFC 3339 string
        ];

        let expected_timestamps = vec![
            "2009-01-03T18:15:05Z", // The expected timestamp, same as test
        ];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1231006505);",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_daylight_saving() {
        // Timestamps just before and after DST change in spring
        let test_timestamps = vec![1583651999, 1583652000]; // Spring forward at 2 AM
        let expected_timestamps = vec![1583651999]; // Only the time before the DST jump should match

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1583651999)",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_leap_year() {
        let test_timestamps = vec![1582934400]; // 2020-02-29T00:00:00Z
        let expected_timestamps = vec![1582934400];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1582934400);",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_time_zone_handling() {
        let test_timestamps = vec![
            1603587600, // 2020-10-25T01:00:00Z in UTC, corresponds to 2 AM in UTC+1 before DST ends
            1603591200, // Corresponds to 2 AM in UTC+1 after DST ends (1 hour later)
        ];
        let expected_timestamps = vec![1603587600];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1603587600)",
            &test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }
}
