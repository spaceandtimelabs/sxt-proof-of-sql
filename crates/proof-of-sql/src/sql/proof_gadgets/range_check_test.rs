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
// A test plan for performing range checks on a specified column.
struct RangeCheckTestPlan {
    // The column reference for the range check test.
    column: ColumnRef,
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
        final_round_evaluate_range_check(builder, scalars, scalars.len(), alloc);
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
                ColumnRef, ColumnType, OwnedTableTestAccessor,
            },
            scalar::Curve25519Scalar,
        },
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;
    use num_bigint::BigUint;
    use num_traits::Num;

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
        let upper_bound_str =
            "452312848583266388373324160190187140051835877600158453279131187530910662655";
        // Parse the number into a BigUint
        let big_uint = BigUint::from_str_radix(upper_bound_str, 10).unwrap();
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0..257)
                .map(|i| upper_bound - Curve25519Scalar::from(i as u64)) // Count backward from 2^248
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
    #[allow(clippy::cast_sign_loss)]
    fn we_can_prove_a_range_check_below_max_word_value() {
        // 2^248 - 1
        let upper_bound_str =
            "452312848583266388373324160190187140051835877600158453279131187530910662655";
        // Parse the number into a BigUint
        let big_uint = BigUint::from_str_radix(upper_bound_str, 10).unwrap();
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0..1)
                .map(|i| upper_bound - Curve25519Scalar::from(i as u64)) // Count backward from 2^248
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
    #[allow(clippy::cast_sign_loss)]
    fn we_cannot_prove_a_range_check_equal_to_range_boundary() {
        // 2^248
        let upper_bound_str =
            "452312848583266388373324160190187140051835877600158453279131187530910662656";
        // Parse the number into a BigUint
        let big_uint = BigUint::from_str_radix(upper_bound_str, 10).unwrap();
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0..1000)
                .map(|i| upper_bound - Curve25519Scalar::from(i as u64)) // Count backward from 2^248
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
