//! Timestamp-related integration tests for the proof-of-sql crate.
#![cfg(feature = "test")]
#![cfg_attr(test, allow(clippy::missing_panics_doc))]
use ark_std::test_rng;
#[cfg(feature = "blitzar")]
use proof_of_sql::base::commitment::InnerProductProof;
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
        PublicParameters, VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[test]
fn we_can_prove_a_basic_query_containing_rfc3339_timestamp_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([
            tinyint("tinyint", [i8::MIN, 0, i8::MAX]),
            smallint("smallint", [i16::MIN, 0, i16::MAX]),
            int("int", [i32::MIN, 0, i32::MAX]),
            bigint("bigint", [i64::MIN, 0, i64::MAX]),
            int128("int128", [i128::MIN, 0, i128::MAX]),
            timestamptz(
                "times",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [i64::MIN, 0, i64::MAX],
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT times FROM table WHERE times = timestamp '1970-01-01T00:00:00Z';"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz(
        "times",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        [0],
    )]);
    assert_eq!(owned_table_result, expected_result);
}

/// Runs a timestamp query test.
#[cfg(feature = "blitzar")]
fn run_timestamp_query_test(
    query_str: &str,
    test_timestamps: Vec<i64>,     // Input timestamps for the test
    expected_timestamps: Vec<i64>, // Expected timestamps to match the query result
) {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());

    // Setting up a table specifically for timestamps
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([timestamptz(
            "times",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            test_timestamps,
        )]),
        0,
    );

    // Parse and execute the query
    let query = QueryExpr::try_new(query_str.parse().unwrap(), "sxt".into(), &accessor).unwrap();

    let proof = VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());

    // Verify the results
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz(
        "times",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        expected_timestamps,
    )]);

    // Check if the results match the expected results
    assert_eq!(owned_table_result, expected_result);
}

#[cfg(feature = "blitzar")]
#[cfg(feature = "test")]
mod tests {

    use crate::run_timestamp_query_test;
    use chrono::DateTime;

    #[test]
    fn test_basic_timestamp_query() {
        let test_timestamps = vec![1_609_459_200, 1_612_137_600, 1_614_556_800];
        let expected_timestamps = vec![1_609_459_200];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2021-01-01T00:00:00Z';",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_basic_timestamp_inequality_query() {
        let test_timestamps = vec![i64::MIN, -1, 0, 1, i64::MAX];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00Z';",
            test_timestamps.clone(),
            vec![i64::MIN, -1],
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00Z';",
            test_timestamps.clone(),
            vec![1, i64::MAX],
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00Z';",
            test_timestamps.clone(),
            vec![0, 1, i64::MAX],
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00Z';",
            test_timestamps.clone(),
            vec![i64::MIN, -1, 0],
        );
    }

    #[test]
    fn test_timestamp_inequality_queries_with_timezone_offsets() {
        // Test with a range of timestamps around the Unix epoch
        // 60 * 60 = 3600 * 8 (PST offset) = 28800
        let test_timestamps = vec![i64::MIN, 28800, 28799, 0, 1, i64::MAX];

        // Test timezone offset -08:00 (e.g., Pacific Standard Time)
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps.clone(),
            vec![i64::MAX],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps.clone(),
            vec![i64::MIN, 28799, 0, 1],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps.clone(),
            vec![28800, i64::MAX],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps.clone(),
            vec![i64::MIN, 28800, 28799, 0, 1],
        );

        // Test timezone offset +00:00 (e.g., UTC)
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps.clone(),
            vec![28800, 28799, 1, i64::MAX],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps.clone(),
            vec![i64::MIN],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps.clone(),
            vec![28800, 28799, 0, 1, i64::MAX],
        );
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps.clone(),
            vec![i64::MIN, 0],
        );
    }

    // This test simulates the following query:
    //
    // 1. Creating a table:
    //    CREATE TABLE test_table(name VARCHAR, mytime TIMESTAMP);
    //
    // 2. Inserting values into the table:
    //    INSERT INTO test_table(name, mytime) VALUES
    //    ('a', '2009-01-03T18:15:05+03:00'),
    //    ('b', '2009-01-03T18:15:05+04:00'),
    //    ('c', '2009-01-03T19:15:05+03:00'),
    //    ('d', '2009-01-03T19:15:05+04:00');
    //
    // 3. Selecting entries where the timestamp matches a specific value:
    //    SELECT * FROM test_table WHERE mytime = '2009-01-03T19:15:05+04:00';
    //
    // This test confirms that timestamp parsing matches that of both postgresql
    // and the gateway.
    #[test]
    fn test_timestamp_queries_match_postgresql_and_gateway() {
        let test_timestamps = vec![1_230_995_705, 1_230_992_105, 1_230_999_305, 1_230_995_705];
        let expected_timestamps = vec![1_230_995_705, 1_230_995_705];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T19:15:05+04:00'",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_leap_seconds_parsing() {
        // Unix time for 1998-12-31T23:59:59 UTC is 915148799
        // Assuming leap second at 1998-12-31T23:59:60 UTC is recognized, it would be 915148799
        // Unix time for 1999-01-01T00:00:00 UTC is 915148800
        let test_timestamps = vec![915_148_799, 915_148_800, 915_148_801];
        let expected_timestamps = [915_148_799, 915_148_800, 915_148_801]; // Expect the leap second to be parsed and matched

        // Test the query to select the leap second
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '1998-12-31T23:59:60Z'",
            test_timestamps.clone(),
            expected_timestamps[0..1].to_vec(),
        );

        // Test the query to select the leap second
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '1999-01-01T00:00:00Z';",
            test_timestamps.clone(),
            expected_timestamps[1..2].to_vec(),
        );
    }

    #[test]
    fn test_new_years_eve_boundary() {
        let test_timestamps = vec![
            DateTime::parse_from_rfc3339("2023-12-31T23:59:59Z")
                .unwrap()
                .timestamp(),
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .timestamp(),
        ];
        let expected_timestamps = vec![test_timestamps[1]]; // Expect only the new year start

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-01-01T00:00:00Z';",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_february_29_leap_year() {
        // Test year 2024 which is a leap year
        let test_timestamps = vec![
            DateTime::parse_from_rfc3339("2024-02-29T12:00:00Z")
                .unwrap()
                .timestamp(),
            DateTime::parse_from_rfc3339("2024-03-01T12:00:00Z")
                .unwrap()
                .timestamp(),
        ];
        let expected_timestamps = vec![test_timestamps[0]]; // Expect the leap day

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-02-29T12:00:00Z';",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_time_zone_crossings() {
        // Checking how the same absolute moment is handled in different time zones
        let test_timestamps = vec![
            DateTime::parse_from_rfc3339("2023-08-15T15:00:00-05:00")
                .unwrap()
                .timestamp(), // Central Time
            DateTime::parse_from_rfc3339("2023-08-15T16:00:00-04:00")
                .unwrap()
                .timestamp(), // Eastern Time, same moment
        ];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2023-08-15T20:00:00Z'", // UTC time
            test_timestamps.clone(),
            test_timestamps,
        );
    }

    #[test]
    fn test_basic_unix_epoch() {
        // Parse the RFC 3339 formatted string to Unix timestamps directly
        let test_timestamps = vec![
            DateTime::parse_from_rfc3339("2009-01-03T18:15:05Z")
                .unwrap()
                .timestamp(), // The test timestamp from RFC 3339 string
        ];

        let expected_timestamps = vec![
            DateTime::parse_from_rfc3339("2009-01-03T18:15:05Z")
                .unwrap()
                .timestamp(), // The expected timestamp, same as test
        ];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1231006505);",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_unix_epoch_daylight_saving() {
        // Timestamps just before and after DST change in spring
        let test_timestamps = vec![1_583_651_999, 1_583_652_000]; // Spring forward at 2 AM
        let expected_timestamps = vec![1_583_651_999]; // Only the time before the DST jump should match

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1583651999)",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_unix_epoch_leap_year() {
        let test_timestamps = vec![1_582_934_400]; // 2020-02-29T00:00:00Z
        let expected_timestamps = vec![1_582_934_400];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1582934400);",
            test_timestamps,
            expected_timestamps,
        );
    }

    #[test]
    fn test_unix_epoch_time_zone_handling() {
        let test_timestamps = vec![
            1_603_587_600, // 2020-10-25T01:00:00Z in UTC, corresponds to 2 AM in UTC+1 before DST ends
            1_603_591_200, // Corresponds to 2 AM in UTC+1 after DST ends (1 hour later)
        ];
        let expected_timestamps = vec![1_603_587_600];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1603587600)",
            test_timestamps,
            expected_timestamps,
        );
    }
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_timestamp_inequality_queries_with_multiple_columns() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
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
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::utc(),
                [
                    i64::MIN,
                    2,
                    -1,
                    0,
                    1,
                    -2,
                    1_231_006_505_000_000_000, // bitcoin genesis block time
                    i64::MAX,
                ],
            ),
            timestamptz(
                "b",
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::utc(),
                [
                    i64::MAX,
                    -2,
                    -1,
                    -1_231_006_505_000_000_000,
                    0,
                    1,
                    2,
                    i64::MIN,
                ],
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "select *, a <= b as res from TABLE where a <= b"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        timestamptz(
            "a",
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            [i64::MIN, -1, -2],
        ),
        timestamptz(
            "b",
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            [i64::MAX, -1, 1],
        ),
        boolean("res", [true, true, true]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}
