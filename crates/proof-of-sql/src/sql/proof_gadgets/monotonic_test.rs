//! This module contains the implementation of the `MonotonicTestPlan` struct. This struct
//! is used to check whether the monotonic gadget works correctly.
use super::monotonic::{
    final_round_evaluate_monotonic, first_round_evaluate_monotonic, verify_monotonic,
};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MonotonicTestPlan<const STRICT: bool, const ASC: bool> {
    pub column: ColumnRef,
}

impl<const STRICT: bool, const ASC: bool> ProverEvaluate for MonotonicTestPlan<STRICT, ASC> {
    #[doc = "Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Get the tables from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let num_rows = table.num_rows();
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(num_rows);
        // Evaluate the first round
        first_round_evaluate_monotonic(builder, num_rows);
        // This is just a dummy table, the actual data is not used
        Table::try_new_with_options(IndexMap::default(), TableOptions { row_count: Some(0) })
            .unwrap()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Get the table from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let raw_column: Vec<S> = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table")
            .to_scalar_with_scaling(0);
        let alloc_column = alloc.alloc_slice_copy(&raw_column);
        builder.produce_intermediate_mle(alloc_column as &[_]);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        final_round_evaluate_monotonic::<S, STRICT, ASC>(builder, alloc, alpha, beta, alloc_column);
        // Return a dummy table
        Table::try_new_with_options(IndexMap::default(), TableOptions { row_count: Some(0) })
            .unwrap()
    }
}

impl<const STRICT: bool, const ASC: bool> ProofPlan for MonotonicTestPlan<STRICT, ASC> {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    #[doc = "Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }

    #[doc = "Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // Get the challenges from the builder
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        // Get evaluations
        let column_eval = builder.try_consume_final_round_mle_evaluation()?;
        let chi_eval = builder.try_consume_chi_evaluation()?;
        // Evaluate the verifier
        verify_monotonic::<S, STRICT, ASC>(builder, alpha, beta, column_eval, chi_eval)?;
        Ok(TableEvaluation::new(vec![], S::zero()))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{table_utility::*, ColumnType, TableTestAccessor},
            math::decimal::Precision,
            scalar::Curve25519Scalar,
        },
        sql::proof::{QueryError, VerifiableQueryResult},
    };
    use blitzar::proof::InnerProductProof;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    fn check_monotonic<const STRICT: bool, const ASC: bool>(
        table_ref: TableRef,
        accessor: &TableTestAccessor<InnerProductProof>,
        column_name: &str,
        column_type: ColumnType,
        shall_error: bool,
    ) {
        let plan = MonotonicTestPlan::<STRICT, ASC> {
            column: ColumnRef::new(table_ref, column_name.into(), column_type),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, accessor, &());
        let res = verifiable_res.verify(&plan, accessor, &());
        if shall_error {
            assert!(matches!(
                res,
                Err(QueryError::ProofError {
                    source: ProofError::VerificationError { .. }
                })
            ));
        } else {
            assert!(res.is_ok());
        }
    }

    /// Monotonicity of a column
    enum Monotonicity {
        /// The column is constant e.g. [1, 1, 1, 1]
        Constant,
        /// The column is strictly increasing e.g. [1, 2, 3, 4]
        StrictlyIncreasing,
        /// The column is increasing but not strictly so and not constant e.g. [1, 1, 2, 3]
        NonStrictlyIncreasing,
        /// The column is strictly decreasing e.g. [4, 3, 2, 1]
        StrictlyDecreasing,
        /// The column is decreasing but not strictly so and not constant e.g. [3, 2, 2, 1]
        NonStrictlyDecreasing,
        /// The column is non-monotonic e.g. [1, 2, 1, 2]
        NonMonotonic,
        /// The column is empty, making all checks vacuously true
        Vacuous,
    }

    impl Monotonicity {
        fn is_strict_asc(&self) -> bool {
            matches!(
                self,
                Monotonicity::StrictlyIncreasing | Monotonicity::Vacuous
            )
        }

        fn is_asc(&self) -> bool {
            matches!(
                self,
                Monotonicity::StrictlyIncreasing
                    | Monotonicity::NonStrictlyIncreasing
                    | Monotonicity::Constant
                    | Monotonicity::Vacuous
            )
        }

        fn is_strict_desc(&self) -> bool {
            matches!(
                self,
                Monotonicity::StrictlyDecreasing | Monotonicity::Vacuous
            )
        }

        fn is_desc(&self) -> bool {
            matches!(
                self,
                Monotonicity::StrictlyDecreasing
                    | Monotonicity::NonStrictlyDecreasing
                    | Monotonicity::Constant
                    | Monotonicity::Vacuous
            )
        }
    }

    /// Run `check_monotonic` for all columns in a table with known data types
    ///
    /// Note that all columns in the table should have the same monotonicity
    /// e.g. constant, strictly increasing, strictly decreasing,
    /// increasing but not strictly increasing, decreasing but non strictly decreasing,
    /// non-monotonic
    fn check_monotonic_for_table<const STRICT: bool, const ASC: bool>(
        table_ref: TableRef,
        accessor: &TableTestAccessor<InnerProductProof>,
        shall_error: bool,
    ) {
        let precision = Precision::new(50).unwrap();
        check_monotonic::<STRICT, ASC>(
            table_ref.clone(),
            accessor,
            "smallint",
            ColumnType::SmallInt,
            shall_error,
        );
        check_monotonic::<STRICT, ASC>(
            table_ref.clone(),
            accessor,
            "int",
            ColumnType::Int,
            shall_error,
        );
        check_monotonic::<STRICT, ASC>(
            table_ref.clone(),
            accessor,
            "bigint",
            ColumnType::BigInt,
            shall_error,
        );
        check_monotonic::<STRICT, ASC>(
            table_ref.clone(),
            accessor,
            "boolean",
            ColumnType::Boolean,
            shall_error,
        );
        check_monotonic::<STRICT, ASC>(
            table_ref.clone(),
            accessor,
            "decimal",
            ColumnType::Decimal75(precision, 1),
            shall_error,
        );
        check_monotonic::<STRICT, ASC>(
            table_ref,
            accessor,
            "timestamp",
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
            shall_error,
        );
    }

    /// Run `check_monotonic_for_table` for all possible forms of monotonicity
    fn check_all_monotonic_for_table(
        table: Table<Curve25519Scalar>,
        expected_monotonicity: &Monotonicity,
    ) {
        let table_ref: TableRef = "sxt.table".parse().unwrap();
        let accessor =
            TableTestAccessor::<InnerProductProof>::new_from_table(table_ref.clone(), table, 0, ());
        check_monotonic_for_table::<true, true>(
            table_ref.clone(),
            &accessor,
            !expected_monotonicity.is_strict_asc(),
        );
        check_monotonic_for_table::<false, true>(
            table_ref.clone(),
            &accessor,
            !expected_monotonicity.is_asc(),
        );
        check_monotonic_for_table::<true, false>(
            table_ref.clone(),
            &accessor,
            !expected_monotonicity.is_strict_desc(),
        );
        check_monotonic_for_table::<false, false>(
            table_ref,
            &accessor,
            !expected_monotonicity.is_desc(),
        );
    }

    #[test]
    fn we_can_check_monotonicity_for_empty_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [0_i16; 0], &alloc),
            borrowed_int("int", [0; 0], &alloc),
            borrowed_bigint("bigint", [0_i64; 0], &alloc),
            borrowed_boolean("boolean", [false; 0], &alloc),
            borrowed_decimal75("decimal", 50, 1, [0; 0], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![0; 0],
                &alloc,
            ),
        ]);
        // Vacuously true
        check_all_monotonic_for_table(table, &Monotonicity::Vacuous);
    }

    #[test]
    fn we_can_check_monotonicity_for_zero_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [0_i16; 3], &alloc),
            borrowed_int("int", [0; 3], &alloc),
            borrowed_bigint("bigint", [0_i64; 3], &alloc),
            borrowed_boolean("boolean", [false; 3], &alloc),
            borrowed_decimal75("decimal", 50, 1, [0; 3], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![0; 3],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::Constant);
    }

    #[test]
    fn we_can_check_monotonicity_for_const_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [1_i16; 3], &alloc),
            borrowed_int("int", [1; 3], &alloc),
            borrowed_bigint("bigint", [-1_i64; 3], &alloc),
            borrowed_boolean("boolean", [true; 3], &alloc),
            borrowed_decimal75("decimal", 50, 1, [1; 3], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_072_400; 3],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::Constant);
    }

    #[test]
    fn we_can_check_monotonicity_for_strictly_increasing_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [i16::MIN, i16::MAX], &alloc),
            borrowed_int("int", [-2, -1], &alloc),
            borrowed_bigint("bigint", [1, 2], &alloc),
            borrowed_boolean("boolean", [false, true], &alloc),
            borrowed_decimal75("decimal", 50, 1, [-1, 1], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_072_400, 1_625_076_000],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::StrictlyIncreasing);
    }

    #[test]
    fn we_can_check_monotonicity_for_increasing_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [1_i16, 2, 2], &alloc),
            borrowed_int("int", [-2, -2, 0], &alloc),
            borrowed_bigint("bigint", [-1, 0, 0], &alloc),
            borrowed_boolean("boolean", [false, false, true], &alloc),
            borrowed_decimal75("decimal", 50, 1, [-1, 1, 1], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_072_400, 1_625_076_000, 1_625_076_000],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::NonStrictlyIncreasing);
    }

    #[test]
    fn we_can_check_monotonicity_for_strictly_decreasing_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [i16::MAX, i16::MIN], &alloc),
            borrowed_int("int", [-1, -2], &alloc),
            borrowed_bigint("bigint", [2, 1], &alloc),
            borrowed_boolean("boolean", [true, false], &alloc),
            borrowed_decimal75("decimal", 50, 1, [1, -1], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_076_000, 1_625_072_400],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::StrictlyDecreasing);
    }

    #[test]
    fn we_can_check_monotonicity_for_decreasing_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [2_i16, 2, 1], &alloc),
            borrowed_int("int", [0, -2, -2], &alloc),
            borrowed_bigint("bigint", [0, 0, -1], &alloc),
            borrowed_boolean("boolean", [true, false, false], &alloc),
            borrowed_decimal75("decimal", 50, 1, [1, 1, -1], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_076_000, 1_625_076_000, 1_625_072_400],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::NonStrictlyDecreasing);
    }

    #[test]
    fn we_can_check_monotonicity_for_non_monotonic_columns() {
        let alloc = Bump::new();
        let table = table([
            borrowed_smallint("smallint", [1_i16, 2, 1], &alloc),
            borrowed_int("int", [-2, -1, -2], &alloc),
            borrowed_bigint("bigint", [1, 0, 1], &alloc),
            borrowed_boolean("boolean", [false, true, false], &alloc),
            borrowed_decimal75("decimal", 50, 1, [-1, 1, -1], &alloc),
            borrowed_timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![1_625_072_400, 1_625_076_000, 1_625_072_400],
                &alloc,
            ),
        ]);
        check_all_monotonic_for_table(table, &Monotonicity::NonMonotonic);
    }
}
