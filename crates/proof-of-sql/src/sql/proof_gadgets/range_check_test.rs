use super::range_check::{
    final_round_evaluate_range_check, first_round_evaluate_range_check,
    verifier_evaluate_range_check,
};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, ColumnType, OwnedTable, Table, TableEvaluation, TableRef,
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
// A test plan for performing range checks on a specified column.
pub struct RangeCheckTestPlan {
    // The column reference for the range check test.
    pub column: ColumnRef,
}

macro_rules! handle_column_with_match {
    ($col:expr, $fn_name:ident, $builder:expr, $alloc:expr) => {
        match $col.column_type() {
            ColumnType::BigInt => {
                let slice = $col
                    .as_bigint()
                    .expect("column_type() is BigInt, but as_bigint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Int => {
                let slice = $col
                    .as_int()
                    .expect("column_type() is Int, but as_int() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::SmallInt => {
                let slice = $col
                    .as_smallint()
                    .expect("column_type() is SmallInt, but as_smallint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::TinyInt => {
                let slice = $col
                    .as_tinyint()
                    .expect("column_type() is TinyInt, but as_tinyint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Uint8 => {
                let slice = $col
                    .as_uint8()
                    .expect("column_type() is Uint8, but as_uint8() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Int128 => {
                let slice = $col
                    .as_int128()
                    .expect("column_type() is Int128, but as_int128() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Decimal75(_precision, _scale) => {
                let slice = $col
                    .as_decimal75()
                    .expect("column_type() is Decimal75, but as_decimal75() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Scalar => {
                let slice = $col
                    .as_scalar()
                    .expect("column_type() is Scalar, but as_scalar() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::TimestampTZ(_tu, _tz) => {
                let slice = $col
                    .as_timestamptz()
                    .expect("column_type() is TimestampTZ, but as_timestamptz() was None");
                $fn_name($builder, slice, $alloc);
            }
            _ => {
                panic!("Unsupported column type in handle_column_with_match");
            }
        }
    };
}

impl ProverEvaluate for RangeCheckTestPlan {
    #[doc = " Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        builder.request_post_result_challenges(1);
        builder.update_range_length(256);

        let table = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");

        // Extract the column data
        let col = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table");

        handle_column_with_match!(col, first_round_evaluate_range_check, builder, alloc);

        builder.produce_one_evaluation_length(256);

        // Return a clone of the same table
        table.clone()
    }

    // extract data to test on from here, feed it into range check
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let col = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table");

        handle_column_with_match!(col, final_round_evaluate_range_check, builder, alloc);

        table.clone()
    }
}

impl ProofPlan for RangeCheckTestPlan {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(
            self.column.column_id(),
            *self.column.column_type(),
        )]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    #[doc = " Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }

    #[doc = " Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_column_eval = accessor[&self.column];
        let input_ones_eval = one_eval_map[&self.column.table_ref()];

        verifier_evaluate_range_check(builder, input_column_eval, input_ones_eval)?;

        Ok(TableEvaluation::new(
            vec![accessor[&self.column]],
            one_eval_map[&self.column.table_ref()],
        ))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{owned_table_utility::*, ColumnRef, ColumnType, OwnedTableTestAccessor},
            math::decimal::Precision,
            scalar::Curve25519Scalar,
        },
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;
    use num_bigint::BigUint;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    fn check_range(
        table_name: TableRef,
        col_name: &str,
        col_type: ColumnType,
        accessor: &OwnedTableTestAccessor<InnerProductProof>,
    ) {
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(table_name, col_name.into(), col_type),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, accessor, &());
        assert!(verifiable_res.verify(&ast, accessor, &()).is_ok());
    }

    #[test]
    fn we_can_prove_ranges_on_mixed_column_types() {
        let data = owned_table([
            uint8("uint8", [0, u8::MAX]),
            tinyint("tinyint", [0, i8::MAX]),
            smallint("smallint", [0, i16::MAX]),
            int("int", [0, i32::MAX]),
            bigint("bigint", [0, i64::MAX]),
            int128("int128", [0, i128::MAX]),
            timestamptz(
                "times",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [0, i64::MAX],
            ),
            decimal75(
                "decimal75",
                74,
                0,
                [
                    Curve25519Scalar::ZERO,
                    // 2^248 - 1
                    Curve25519Scalar::from_bigint(
                        (BigUint::from(2u8).pow(248) - BigUint::from(1u8))
                            .to_u64_digits()
                            .try_into()
                            .unwrap(),
                    ),
                ],
            ),
        ]);

        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());

        check_range(t, "uint8", ColumnType::Uint8, &accessor);
        check_range(t, "tinyint", ColumnType::TinyInt, &accessor);
        check_range(t, "smallint", ColumnType::SmallInt, &accessor);
        check_range(t, "int", ColumnType::Int, &accessor);
        check_range(t, "bigint", ColumnType::BigInt, &accessor);
        check_range(t, "int128", ColumnType::Int128, &accessor);
        check_range(
            t,
            "decimal75",
            ColumnType::Decimal75(Precision::new(74).unwrap(), 0),
            &accessor,
        );
        check_range(
            t,
            "times",
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
            &accessor,
        );
    }

    #[test]
    #[should_panic(
        expected = "Range check failed, column contains values outside of the selected range"
    )]
    fn we_cannot_successfully_verify_invalid_range() {
        let data = owned_table([scalar("a", -2..254)]);
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let _ = verifiable_res.verify(&ast, &accessor, &());
    }

    #[test]
    #[allow(clippy::cast_sign_loss)]
    fn we_can_prove_a_range_check_with_range_up_to_boundary() {
        // 2^248 - 1
        let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0..2u32.pow(10))
                .map(|i| upper_bound - Curve25519Scalar::from(u64::from(i))) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let res: Result<
            crate::sql::proof::QueryData<crate::base::scalar::MontScalar<ark_curve25519::FrConfig>>,
            crate::sql::proof::QueryError,
        > = verifiable_res.verify(&ast, &accessor, &());

        if let Err(e) = res {
            panic!("Verification failed: {e}");
        }
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_prove_a_range_check_with_range_below_max_word_value() {
        // 2^248 - 1
        let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
        // Parse the number into a BigUint
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0u8..1)
                .map(|i| upper_bound - Curve25519Scalar::from(i)) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let res: Result<
            crate::sql::proof::QueryData<crate::base::scalar::MontScalar<ark_curve25519::FrConfig>>,
            crate::sql::proof::QueryError,
        > = verifiable_res.verify(&ast, &accessor, &());

        if let Err(e) = res {
            panic!("Verification failed: {e}");
        }
        assert!(res.is_ok());
    }

    #[test]
    #[should_panic(
        expected = "Range check failed, column contains values outside of the selected range"
    )]
    fn we_cannot_prove_a_range_check_equal_to_range_boundary() {
        // 2^248
        let big_uint = BigUint::from(2u8).pow(248);
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0u16..2u16.pow(10))
                .map(|i| upper_bound - Curve25519Scalar::from(i)) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        verifiable_res.verify(&ast, &accessor, &()).unwrap();
    }
}
