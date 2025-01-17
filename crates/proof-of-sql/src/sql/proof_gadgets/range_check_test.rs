use super::range_check::{
    final_round_evaluate_range_check, first_round_evaluate_range_check,
    verifier_evaluate_range_check,
};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableRef},
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
/// A test plan for performing range checks on a specified column.
pub struct RangeCheckTestPlan {
    /// The column reference for the range check test.
    pub column: ColumnRef,
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

        // Get the table from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");

        let scalars = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table")
            .as_scalar()
            .expect("Failed to convert column to scalar");

        first_round_evaluate_range_check(builder, scalars, alloc);

        builder.produce_one_evaluation_length(256);

        table_map[&self.column.table_ref()].clone()
    }

    // extract data to test on from here, feed it into range check
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

        let scalars = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table")
            .as_scalar()
            .expect("Failed to convert column to scalar");
        final_round_evaluate_range_check(builder, scalars, alloc);
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
            database::{
                owned_table_utility::{owned_table, scalar},
                ColumnRef, ColumnType, OwnedTableTestAccessor, TestAccessor,
            },
            scalar::Curve25519Scalar,
        },
        proof_primitive::dory::{
            DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
        },
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;
    use num_bigint::BigUint;
    use std::path::Path;

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
            (0..2u32.pow(20))
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
            (0u16..1000)
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

    #[test]
    #[ignore]
    // Ignore in the CI because wont have access to files. Run this with:
    // cargo test sql::proof_gadgets::range_check_test::tests::testing_dory_range_check_boundary_conditions --release  -- --nocapture --ignored
    fn testing_dory_range_check_boundary_conditions() {
        let blitzar_handle_path = std::env::var("BLITZAR_HANDLE_PATH")
            .expect("Environment variable BLITZAR_HANDLE_PATH not set");
        let public_parameters_path = std::env::var("PUBLIC_PARAMETERS_PATH")
            .expect("Environment variable PUBLIC_PARAMETERS_PATH not set");
        let verifier_setup_path = std::env::var("VERIFIER_SETUP_PATH")
            .expect("Environment variable VERIFIER_SETUP_PATH not set");

        let handle = blitzar::compute::MsmHandle::new_from_file(&blitzar_handle_path);
        let public_parameters =
            PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

        let prover_setup =
            ProverSetup::from_public_parameters_and_blitzar_handle(&public_parameters, handle);
        let verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
            .expect("Failed to load VerifierSetup");

        // 2^248 - 1
        let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = DoryScalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<DoryScalar> = owned_table([scalar(
            "a",
            (0..2u32.pow(20))
                .map(|i| upper_bound - DoryScalar::from(u64::from(i))) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = "sxt.t".parse().unwrap();
        let mut accessor =
            OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(
                &prover_setup,
            );

        accessor.add_table("sxt.t".parse().unwrap(), data, 0);

        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };

        let verifiable_res = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
            &ast,
            &accessor,
            &&prover_setup,
        );

        let res = verifiable_res.verify(&ast, &accessor, &&verifier_setup);

        if let Err(e) = res {
            panic!("Verification failed: {e}");
        }
        assert!(res.is_ok());
    }
}
