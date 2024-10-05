#![cfg(feature = "test")]
#![cfg_attr(test, allow(clippy::missing_panics_doc))]
#[cfg(feature = "blitzar")]
use blitzar::proof::InnerProductProof;
#[cfg(feature = "blitzar")]
use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar as S};
use proof_of_sql::sql::postprocessing::apply_postprocessing_steps;

#[cfg(feature = "blitzar")]
fn run_query(
    query_str: &str,
    expected_precision: u8,
    expected_scale: i8,
    test_decimal_values: Vec<S>,
    expected_decimal_values: Vec<S>,
) {
    use proof_of_sql::{
        base::database::{OwnedTable, OwnedTableTestAccessor, TestAccessor},
        sql::{parse::QueryExpr, proof::VerifiableQueryResult},
    };

    // Setup common data and accessor for each run
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let data: OwnedTable<S> = owned_table([
        varchar("b", ["t", "u", "v"]),
        bigint("a", [1, 2, 3]),
        decimal75("c", expected_precision, expected_scale, test_decimal_values),
    ]);

    accessor.add_table("sxt.table".parse().unwrap(), data, 0);

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
    let owned_table_result: OwnedTable<_> =
        apply_postprocessing_steps(owned_table_result, query.postprocessing()).unwrap();

    // Adjust expected result based on the precision and scale provided
    let expected_result = owned_table::<S>([
        varchar("b", ["t", "v"]),
        bigint("a", [1, 3]),
        decimal75(
            "c",
            expected_precision,
            expected_scale,
            expected_decimal_values,
        ),
    ]);
    // Verify the result matches the expectation
    assert_eq!(owned_table_result, expected_result);
}

#[cfg(feature = "blitzar")]
mod decimal_query_tests {
    use crate::run_query;
    use proof_of_sql::base::scalar::{Curve25519Scalar as S, Scalar};

    #[test]
    fn we_can_query_decimals_exactly_matching_db_data() {
        run_query(
            "SELECT * FROM table WHERE c = 1.0;",
            2,
            0,
            vec![S::from(1), S::ZERO, S::ONE],
            vec![S::from(1), S::ONE],
        );
    }

    #[test]
    fn we_can_query_simple_decimals() {
        run_query(
            "SELECT * FROM table WHERE c = 1.1",
            2,
            1,
            vec![S::from(11), S::ZERO, S::from(11)],
            vec![S::from(11), S::from(11)],
        );
    }

    #[test]
    fn we_can_query_negative_valued_decimals_exactly_matching_db_data() {
        run_query(
            "SELECT * FROM table WHERE c = -1.0;",
            2,
            0,
            vec![S::from(-1), S::ZERO, -S::ONE],
            vec![S::from(-1), -S::ONE],
        );
    }

    #[test]
    fn we_can_query_zero_as_decimal() {
        run_query(
            "SELECT * FROM table WHERE c = 0.0",
            2,
            1,
            vec![S::ZERO, S::ONE, S::ZERO],
            vec![S::ZERO, S::ZERO],
        );
    }

    #[test]
    fn we_can_query_negative_decimals_with_different_scale_than_db_data() {
        run_query(
            "SELECT * FROM table WHERE c = -1.0;",
            4,
            2,
            vec![S::from(-100), S::ZERO, S::from(-100)],
            vec![S::from(-100), S::from(-100)],
        );
    }

    #[test]
    fn we_can_query_with_negative_values_with_trailing_zeros() {
        run_query(
            "SELECT * FROM table WHERE c = -1.000;",
            4,
            2,
            vec![S::from(-100), S::ZERO, S::from(-100)],
            vec![S::from(-100), S::from(-100)],
        );
    }

    #[test]
    fn we_can_query_decimals_without_leading_zeros() {
        run_query(
            "SELECT * FROM table WHERE c = .1",
            2,
            1,
            vec![S::from(1), S::ZERO, S::ONE],
            vec![S::from(1), S::ONE],
        );
    }

    #[test]
    fn we_can_query_decimals_with_leading_zeros() {
        run_query(
            "SELECT * FROM table WHERE c = 0.1;",
            1,
            1,
            vec![S::from(1), S::ZERO, S::ONE],
            vec![S::from(1), S::ONE],
        );
    }

    #[test]
    fn we_can_query_decimals_with_lower_scale_than_db_data() {
        run_query(
            "SELECT * FROM table WHERE c = 0.1",
            7,
            6,
            vec![S::from(100000), S::ZERO, S::from(100000)],
            vec![S::from(100000), S::from(100000)],
        );
    }

    #[test]
    fn we_can_query_negative_decimals_with_trailing_zeros_exactly_matching_db_data() {
        run_query(
            "SELECT * FROM table WHERE c = -0.100000",
            7,
            1,
            vec![S::from(-1), S::ZERO, -S::ONE],
            vec![S::from(-1), -S::ONE],
        );
    }

    #[test]
    fn we_can_query_with_varying_scale_and_precision() {
        run_query(
            "SELECT * FROM table WHERE c = 123.456;",
            6,
            3,
            vec![S::from(123456), S::ZERO, S::from(123456)],
            vec![S::from(123456), S::from(123456)],
        );
    }

    #[test]
    fn we_can_query_integers_against_decimals() {
        run_query(
            "SELECT * FROM table WHERE c = 12345",
            7,
            2,
            vec![S::from(1234500), S::ZERO, S::from(1234500)],
            vec![S::from(1234500), S::from(1234500)],
        );
    }

    #[test]
    fn we_can_query_negative_integers_against_decimals() {
        run_query(
            "SELECT * FROM table WHERE c = -12345",
            7,
            2,
            vec![-S::from(1234500), S::ZERO, -S::from(1234500)],
            vec![-S::from(1234500), -S::from(1234500)],
        );
    }

    #[test]
    fn we_can_query_with_maximum_i64() {
        run_query(
            &format!("SELECT * FROM table WHERE c = {}.0;", i64::MAX),
            75,
            0,
            vec![S::from(i64::MAX), S::ZERO, S::from(i64::MAX)],
            vec![S::from(i64::MAX), S::from(i64::MAX)],
        );
    }

    #[test]
    fn we_can_query_with_maximum_i128() {
        run_query(
            &format!("SELECT * FROM table WHERE c = {}.0;", i128::MAX),
            75,
            0,
            vec![S::from(i128::MAX), S::ZERO, S::from(i128::MAX)],
            vec![S::from(i128::MAX), S::from(i128::MAX)],
        );
    }
}
