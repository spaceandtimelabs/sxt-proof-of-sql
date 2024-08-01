#![cfg(feature = "test")]
#[cfg(feature = "blitzar")]
use proof_of_sql::base::commitment::InnerProductProof;
use proof_of_sql::{
    base::database::{owned_table_utility::*, OwnedTableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        test_rng, DoryCommitment, DoryEvaluationProof, DoryProverPublicSetup,
        DoryVerifierPublicSetup, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{
        parse::QueryExpr,
        proof::{QueryProof, VerifiableQueryResult},
    },
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[test]
#[cfg(feature = "blitzar")]
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
                [
                    "1969-12-31T23:59:59Z", // One second before the Unix epoch
                    "1970-01-01T00:00:00Z", // The Unix epoch
                    "1970-01-01T00:00:01Z", // One second after the Unix epoch
                ]
                .iter()
                .map(|s| s.to_string()),
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
        ["1970-01-01T00:00:00Z".to_string()],
    )]);
    assert_eq!(owned_table_result, expected_result);
}

/// Runs a timestamp query test with unix epochs as input.
#[cfg(feature = "blitzar")]
fn run_timestamp_epoch_query_test(
    query_str: &str,
    test_timestamps: &[i64], // Input timestamps for the test
    test_timeunit: PoSQLTimeUnit,
    expected_timestamps: &[i64], // Expected timestamps to match the query result
    expected_timeunit: PoSQLTimeUnit,
) {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());

    // Setting up a table specifically for timestamps
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([timestamptz_epoch(
            "times",
            test_timeunit,
            PoSQLTimeZone::Utc,
            test_timestamps.to_owned(),
        )]),
        0,
    );

    // Parse and execute the query
    let query = QueryExpr::try_new(
        query_str.parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();

    let proof = VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());

    // Verify the results
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz_epoch(
        "times",
        expected_timeunit,
        PoSQLTimeZone::Utc,
        expected_timestamps.to_owned(),
    )]);

    // Check if the results match the expected results
    assert_eq!(owned_table_result, expected_result);
}

/// Runs a timestamp query test with unix epochs as input.
#[cfg(feature = "blitzar")]
fn run_timestamp_query_test(
    query_str: &str,
    test_timestamps: &[&str], // Input timestamps for the test
    test_timeunit: PoSQLTimeUnit,
    expected_timestamps: &[&str], // Expected timestamps to match the query
    expected_timeunit: PoSQLTimeUnit,
) {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());

    // Setting up a table specifically for timestamps
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([timestamptz(
            "times",
            test_timeunit,
            test_timestamps.iter().map(|s| s.to_string()),
        )]),
        0,
    );

    // Parse and execute the query
    let query = QueryExpr::try_new(
        query_str.parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();

    let proof = VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());

    // Verify the results
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz(
        "times",
        expected_timeunit,
        expected_timestamps.iter().map(|s| s.to_string()),
    )]);

    // Check if the results match the expected results
    assert_eq!(owned_table_result, expected_result);
}

#[cfg(feature = "blitzar")]
#[cfg(feature = "test")]
mod tests {

    use crate::{run_timestamp_epoch_query_test, run_timestamp_query_test};
    use chrono::DateTime;
    use proof_of_sql_parser::posql_time::PoSQLTimeUnit;

    #[test]
    fn test_basic_timestamp_query() {
        let test_timestamps = &[1609459200, 1612137600, 1614556800];
        let expected_timestamps = &[1609459200_i64];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '2021-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[should_panic] //these should pass once the scaling bug is resolved
    #[test]
    fn test_precision_and_rounding() {
        // Testing timestamps near rounding thresholds in milliseconds
        let test_timestamps = &["2009-01-03T18:15:05.999Z"];
        let expected_timestamps = &["2009-01-03T18:15:05.999Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );

        // test microseconds
        let test_timestamps = &["2009-01-03T18:15:05.999999Z"];
        let expected_timestamps = &["2009-01-03T18:15:05.999999Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999999Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );

        // test nanoseconds
        let test_timestamps = &["2009-01-03T18:15:05.999999999Z"];
        let expected_timestamps = &["2009-01-03T18:15:05.999999999Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.999999999Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );

        // test nanoseconds
        let test_timestamps = &["2009-01-03T18:15:05.999Z", "2009-01-03T18:15:05.000Z"];
        let expected_timestamps = &["2009-01-03T18:15:05.000Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[should_panic] //these should pass once the scaling bug is resolved
    #[test]
    fn test_precision_and_rounding_with_differing_precisions() {
        // Testing timestamps near rounding thresholds in milliseconds
        let test_timestamps = &[
            "2009-01-03T18:15:05.999999999Z",
            "2009-01-03T18:15:05.000000000Z",
        ];
        let expected_timestamps = &["2009-01-03T18:15:05.000000000Z"];
        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
            test_timestamps,
            PoSQLTimeUnit::Nanosecond,
            expected_timestamps,
            PoSQLTimeUnit::Nanosecond,
        );

        // // Testing timestamps near rounding thresholds in milliseconds
        // let test_timestamps = &["2009-01-03T18:15:05.999999Z", "2009-01-03T18:15:05.000000Z"];
        // let expected_timestamps = &["2009-01-03T18:15:05.000000Z"];
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        //     test_timestamps,
        //     expected_timestamps,
        // );

        // // Testing timestamps near rounding thresholds in milliseconds
        // let test_timestamps = &["2009-01-03T18:15:05.999Z", "2009-01-03T18:15:05.000Z"];
        // let expected_timestamps = &["2009-01-03T18:15:05.000Z"];
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        //     test_timestamps,
        //     expected_timestamps,
        // );

        // // Test scaling a query literal to match a variety of timestamp precisions
        // let test_timestamps = &[
        //     "2009-01-03T18:15:05.0Z",
        //     "2009-01-03T18:15:05.00Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.0000Z",
        //     "2009-01-03T18:15:05.00000Z",
        //     "2009-01-03T18:15:05.000000Z",
        //     "2009-01-03T18:15:05.0000000Z",
        //     "2009-01-03T18:15:05.00000000Z",
        //     "2009-01-03T18:15:05.000000000Z",
        //     "2009-01-03T18:15:05Z",
        //     "2009-01-03T18:15:05.1Z",
        //     "2009-01-03T18:15:05.12Z",
        //     "2009-01-03T18:15:05.123Z",
        //     "2009-01-03T18:15:05.1234Z",
        //     "2009-01-03T18:15:05.12345Z",
        //     "2009-01-03T18:15:05.123456Z",
        //     "2009-01-03T18:15:05.1234567Z",
        //     "2009-01-03T18:15:05.1234568Z",
        //     "2009-01-03T18:15:05.12345689Z",
        // ];
        // let expected_timestamps = &[
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        //     "2009-01-03T18:15:05.000Z",
        // ];
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
        //     test_timestamps,
        //     expected_timestamps,
        // );
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05.123456Z';",
        //     test_timestamps,
        //     &["2009-01-03T18:15:05.123456Z"],
        // );
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times > timestamp '2009-01-03T18:15:05.123456Z';",
        //     test_timestamps,
        //     &[
        //         "2009-01-03T18:15:05.1234567Z",
        //         "2009-01-03T18:15:05.1234568Z",
        //         "2009-01-03T18:15:05.12345689Z",
        //     ],
        // );
        // run_timestamp_query_test(
        //     "SELECT * FROM table WHERE times < timestamp '2009-01-03T18:15:05.123456Z';",
        //     test_timestamps,
        //     &[
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05.000Z",
        //         "2009-01-03T18:15:05Z",
        //         "2009-01-03T18:15:05.1Z",
        //         "2009-01-03T18:15:05.12Z",
        //         "2009-01-03T18:15:05.123Z",
        //         "2009-01-03T18:15:05.1234Z",
        //         "2009-01-03T18:15:05.12345Z",
        //     ],
        // );
    }

    #[test]
    fn test_equality_with_variety_of_rfc3339_timestamps() {
        // Testing timestamps near rounding thresholds
        let test_timestamps = &[
            "2009-01-03T18:15:05Z", // Bitcoin genesis block time
            "1970-01-01T00:00:00Z", // Unix epoch
            "1969-07-20T20:17:40Z", // Apollo 11 moon landing
            "1993-04-30T00:00:00Z", // World Wide Web goes live
            "1927-03-07T00:00:00Z", // Discovery of Penicillin
            "2004-02-04T00:00:00Z", // Founding of Facebook
            "2011-11-26T05:17:57Z", // Curiosity Rover lands on Mars
        ];
        let expected_timestamps = &["2009-01-03T18:15:05Z"];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T18:15:05Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1993-04-30T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[
                "2009-01-03T18:15:05Z",
                "1993-04-30T00:00:00Z",
                "2004-02-04T00:00:00Z",
                "2011-11-26T05:17:57Z",
            ],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times > timestamp '1993-04-30T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[
                "2009-01-03T18:15:05Z",
                "2004-02-04T00:00:00Z",
                "2011-11-26T05:17:57Z",
            ],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1993-04-30T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1993-04-30T00:00:00Z",
                "1927-03-07T00:00:00Z",
            ],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times < timestamp '1993-04-30T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[
                "1970-01-01T00:00:00Z",
                "1969-07-20T20:17:40Z",
                "1927-03-07T00:00:00Z",
            ],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_basic_timestamp_inequality_query() {
        let test_timestamps = &[i64::MIN, -1, 0, 1, i64::MAX];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[i64::MIN, -1],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[1, i64::MAX],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[0, 1, i64::MAX],
            PoSQLTimeUnit::Second,
        );

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[i64::MIN, -1, 0],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_timestamp_inequality_queries_with_timezone_offsets() {
        // Test with a range of timestamps around the Unix epoch
        // 60 * 60 = 3600 * 8 (PST offset) = 28800
        let test_timestamps = &[28800, 28799, -1, 0, 1];

        // Test timezone offset -08:00 (e.g., Pacific Standard Time)
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[28799, -1, 0, 1],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[28800],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00-08:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[28800, 28799, -1, 0, 1],
            PoSQLTimeUnit::Second,
        );

        // Test timezone offset +00:00 (e.g., UTC)
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times > timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[28800, 28799, 1],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times < timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[-1],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times >= timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[28800, 28799, 0, 1],
            PoSQLTimeUnit::Second,
        );
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times <= timestamp '1970-01-01T00:00:00+00:00';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[-1, 0],
            PoSQLTimeUnit::Second,
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
        let test_timestamps = &[1230995705, 1230992105, 1230999305, 1230995705];
        let expected_timestamps = &[1230995705, 1230995705];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '2009-01-03T19:15:05+04:00'",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_leap_seconds_parsing() {
        // Unix time for 1998-12-31T23:59:59 UTC is 915148799
        // Assuming leap second at 1998-12-31T23:59:60 UTC is recognized, it would be 915148799
        // Unix time for 1999-01-01T00:00:00 UTC is 915148800
        let test_timestamps = &[915148799, 915148800, 915148801];
        let expected_timestamps = [915148799, 915148800, 915148801]; // Expect the leap second to be parsed and matched

        // Test the query to select the leap second
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '1998-12-31T23:59:60Z'",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &[915148799],
            PoSQLTimeUnit::Second,
        );

        // Test the query to select the leap second
        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '1999-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            &expected_timestamps[1..2],
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_new_years_eve_boundary() {
        let test_timestamps = &[
            DateTime::parse_from_rfc3339("2023-12-31T23:59:59Z")
                .unwrap()
                .timestamp(),
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .timestamp(),
        ];
        let expected_timestamps = &[test_timestamps[1]]; // Expect only the new year start

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-01-01T00:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[should_panic] //these should pass once the scaling bug is resolved
    #[test]
    fn test_fractional_seconds_handling() {
        let test_timestamps = &[
            "2023-07-01T12:00:00.999Z", /* "2023-07-01T12:00:01.000Z"*/
        ];
        let expected_timestamps = &["2023-07-01T12:00:00.999Z"];

        run_timestamp_query_test(
            "SELECT * FROM table WHERE times = timestamp '2023-07-01T12:00:00.999Z'",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_february_29_leap_year() {
        // Test year 2024 which is a leap year
        let test_timestamps = &[
            DateTime::parse_from_rfc3339("2024-02-29T12:00:00Z")
                .unwrap()
                .timestamp(),
            DateTime::parse_from_rfc3339("2024-03-01T12:00:00Z")
                .unwrap()
                .timestamp(),
        ];
        let expected_timestamps = &[test_timestamps[0]]; // Expect the leap day

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '2024-02-29T12:00:00Z';",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_time_zone_crossings() {
        // Checking how the same absolute moment is handled in different time zones
        let test_timestamps = &[
            DateTime::parse_from_rfc3339("2023-08-15T15:00:00-05:00")
                .unwrap()
                .timestamp(), // Central Time
            DateTime::parse_from_rfc3339("2023-08-15T16:00:00-04:00")
                .unwrap()
                .timestamp(), // Eastern Time, same moment
        ];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = timestamp '2023-08-15T20:00:00Z'", // UTC time
            test_timestamps,
            PoSQLTimeUnit::Second,
            test_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_basic_unix_epoch() {
        // Parse the RFC 3339 formatted string to Unix timestamps directly
        let test_timestamps = &[
            DateTime::parse_from_rfc3339("2009-01-03T18:15:05Z")
                .unwrap()
                .timestamp(), // The test timestamp from RFC 3339 string
        ];

        let expected_timestamps = &[
            DateTime::parse_from_rfc3339("2009-01-03T18:15:05Z")
                .unwrap()
                .timestamp(), // The expected timestamp, same as test
        ];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1231006505);",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_daylight_saving() {
        // Timestamps just before and after DST change in spring
        let test_timestamps = &[1583651999, 1583652000]; // Spring forward at 2 AM
        let expected_timestamps = &[1583651999]; // Only the time before the DST jump should match

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1583651999)",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_leap_year() {
        let test_timestamps = &[1582934400]; // 2020-02-29T00:00:00Z
        let expected_timestamps = &[1582934400];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1582934400);",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }

    #[test]
    fn test_unix_epoch_time_zone_handling() {
        let test_timestamps = &[
            1603587600, // 2020-10-25T01:00:00Z in UTC, corresponds to 2 AM in UTC+1 before DST ends
            1603591200, // Corresponds to 2 AM in UTC+1 after DST ends (1 hour later)
        ];
        let expected_timestamps = &[1603587600];

        run_timestamp_epoch_query_test(
            "SELECT * FROM table WHERE times = to_timestamp(1603587600)",
            test_timestamps,
            PoSQLTimeUnit::Second,
            expected_timestamps,
            PoSQLTimeUnit::Second,
        );
    }
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_timestamp_inequality_queries_with_multiple_columns() {
    use proof_of_sql_parser::posql_time::PoSQLTimeUnit;

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
                [
                    "2009-01-03T18:15:05Z", // Bitcoin genesis block time
                    "1961-04-12T06:07:00Z", // First human spaceflight by Yuri Gagarin
                    "1969-07-20T20:17:40Z", // Apollo 11 moon landing
                    "1983-01-01T00:00:00Z", // Official start of the Internet (TCP/IP)
                    "1927-03-07T00:00:00Z", // Discovery of Penicillin
                    "2004-02-04T00:00:00Z", // Founding of Facebook
                    "1964-05-20T00:00:00Z", // Cosmic Microwave Background Radiation discovered
                ]
                .iter()
                .map(|s| s.to_string()),
            ),
            timestamptz(
                "b",
                PoSQLTimeUnit::Second,
                [
                    "1953-02-28T00:00:00Z", // Publication of DNA's double helix structure
                    "1970-01-01T00:00:00Z", // Unix epoch
                    "1954-12-23T00:00:00Z", // First successful kidney transplant
                    "1993-04-30T00:00:00Z", // World Wide Web goes live
                    "1905-11-21T00:00:00Z", // Einstein's paper on mass-energy equivalence, E=mc²
                    "2003-04-14T00:00:00Z", // Completion of the first draft of the human genome
                    "2011-11-26T05:17:57Z", // Curiosity Rover lands on Mars
                ]
                .iter()
                .map(|s| s.to_string()),
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
            [
                "1961-04-12T06:07:00Z",
                "1983-01-01T00:00:00Z",
                "1964-05-20T00:00:00Z",
            ]
            .iter()
            .map(|s| s.to_string()),
        ),
        timestamptz(
            "b",
            PoSQLTimeUnit::Second,
            [
                "1970-01-01T00:00:00Z",
                "1993-04-30T00:00:00Z",
                "2011-11-26T05:17:57Z",
            ]
            .iter()
            .map(|s| s.to_string()),
        ),
        boolean("res", [true, true, true]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}
