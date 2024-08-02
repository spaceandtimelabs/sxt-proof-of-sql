use crate::{
    base::database::{
        owned_table_utility::{convert_timestamps_to_epochs, *},
        OwnedTableTestAccessor, TestAccessor,
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
    to_epochs,
};
use blitzar::proof::InnerProductProof;
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimestamp};

/// Functions to convert arbitrary-type slices into unix epoch slices
pub trait TimestampData {
    /// Convert supported sliced types into slices of unix epochs with
    /// a given precision
    fn to_timestamps(&self, timeunit: PoSQLTimeUnit) -> Vec<i64>;
}

impl TimestampData for Vec<i64> {
    fn to_timestamps(&self, timeunit: PoSQLTimeUnit) -> Vec<i64> {
        self.iter()
            .map(|&s| {
                let ts = PoSQLTimestamp::to_timestamp(s).unwrap();
                match timeunit {
                    PoSQLTimeUnit::Second => ts.timestamp.timestamp(),
                    PoSQLTimeUnit::Millisecond => ts.timestamp.timestamp_millis(),
                    PoSQLTimeUnit::Microsecond => ts.timestamp.timestamp_micros(),
                    PoSQLTimeUnit::Nanosecond => ts.timestamp.timestamp_nanos_opt().unwrap(),
                }
            })
            .collect()
    }
}

impl TimestampData for Vec<&str> {
    fn to_timestamps(&self, time_unit: PoSQLTimeUnit) -> Vec<i64> {
        to_epochs!(&self, time_unit)
    }
}

/// Given either a slice of i64 unix epochs, or a slice of rfc3339 strings,
/// this function abstracts away the process of carrying out query against
/// an owned table.
pub fn run_timestamp_query_test<T: TimestampData, U: TimestampData>(
    query_str: &str,
    test_timestamps: &T,
    test_timeunit: PoSQLTimeUnit,
    expected_timestamps: &U,
    expected_timeunit: PoSQLTimeUnit,
) {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());

    let test_ts_vec = test_timestamps.to_timestamps(test_timeunit);
    let expected_ts_vec = expected_timestamps.to_timestamps(expected_timeunit);

    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([timestamptz("times", test_timeunit, test_ts_vec)]),
        0,
    );

    let query = QueryExpr::try_new(
        query_str.parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();

    let proof = VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());

    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([timestamptz("times", expected_timeunit, expected_ts_vec)]);

    assert_eq!(owned_table_result, expected_result);
}
