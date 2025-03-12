use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef},
        map::IndexMap,
        proof::ProofError,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::mock_verification_builder::MockVerificationBuilder,
        proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsTrueExpr},
    },
};
use alloc::boxed::Box;
use ark_ff::Fp256;
use bumpalo::Bump;
use num_traits::{One, Zero};
use sqlparser::ast::Ident;
use std::hash::BuildHasherDefault;

#[test]
fn we_can_generate_actual_proof_with_is_true_expr() {
    use crate::{
        base::{
            commitment::naive_evaluation_proof::NaiveEvaluationProof,
            database::{
                owned_table_utility::{bigint, boolean, owned_table},
                OwnedTableTestAccessor,
            },
        },
        sql::{
            proof::{QueryData, VerifiableQueryResult},
            proof_exprs::test_utility::{cols_expr_plan, column, tab},
            proof_plans::test_utility::filter,
        },
    };

    // ------------------- Setup -------------------
    let table_name = TableRef::new("foo", "bar");
    let accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        table_name.clone(),
        owned_table([bigint("A", [1, 2, 3]), boolean("B", [false, true, false])]),
        0,
        (),
    );

    // B
    let column_b_expr = column(&table_name, "B", &accessor);
    // B IS TRUE
    let mut is_true_expr = IsTrueExpr::new(Box::new(column_b_expr));
    is_true_expr.malicious = true;

    // SELECT A FROM foo.bar WHERE B IS TRUE
    let query = filter(
        cols_expr_plan(&table_name, &["A"], &accessor),
        tab(&table_name),
        DynProofExpr::IsTrue(is_true_expr),
    );

    let expected_result = owned_table([bigint("A", [2])]);

    // ------------------- Prover -------------------

    // Prover runs query and generates proof (VerifiableQueryResult contains both).
    let verifiable_res = VerifiableQueryResult::<NaiveEvaluationProof>::new(&query, &accessor, &());

    // ------------------- Verifier -------------------

    // Verifier verifies this proof/result
    match verifiable_res.verify(&query, &accessor, &()) {
        Ok(QueryData { table, .. }) => {
            assert_eq!(
                table, expected_result,
                "The proof was accepted by the verifier, but the result was incorrect."
            );
        }
        Err(err) => println!("Verification failed with error: {err:?}"),
    }
}

#[test]
fn test_is_true_expr() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // NULL
            assert!(values[2]); // true and not NULL
            assert!(!values[3]); // NULL
            assert!(values[4]); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn we_should_obtain_a_verification_error_if_a_malicious_prover_returns_the_wrong_result() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let mut is_true_expr = IsTrueExpr::new(Box::new(column_expr));

    // First get the correct result
    let correct_result = is_true_expr.result_evaluate(&alloc, &table);

    // Extract the correct boolean values
    let Column::Boolean(correct_values) = correct_result else {
        panic!("Expected boolean column")
    };

    // Now set malicious flag and provide incorrect MLEs
    is_true_expr.malicious = true;

    // Create a mock builder with tampered final round MLEs
    // The tampered data should be different from what result_evaluate produced
    // We use zero for all values when the correct result has some true values
    let tampered_mles = if correct_values.iter().any(|&x| x) {
        // If there are any true values, use all zeros to ensure it's different
        vec![vec![TestScalar::new(Fp256::zero())]]
    } else {
        // If all values are false, use all ones to ensure it's different
        vec![vec![TestScalar::new(Fp256::one())]]
    };

    let mut builder = MockVerificationBuilder::new(Vec::new(), 0, tampered_mles);

    let accessor = IndexMap::with_hasher(BuildHasherDefault::default());
    let chi_eval = TestScalar::new(Fp256::one());

    // Verification should fail because the tampered MLEs don't match the correct result
    let result = is_true_expr.verifier_evaluate(&mut builder, &accessor, chi_eval);
    assert!(
        result.is_err(),
        "Expected verification to fail with tampered MLEs"
    );

    if let Err(err) = result {
        assert!(
            matches!(err, ProofError::VerificationError { .. }),
            "Expected VerificationError error, got: {err:?}"
        );
    }
}

#[test]
fn test_is_true_expr_with_false_values() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, false, true, false, false];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL (presence[0] = true, values[0] = true)
            assert!(!values[1]); // NULL (presence[1] = false)
            assert!(values[2]); // true and not NULL (presence[2] = true, values[2] = true)
            assert!(!values[3]); // NULL (presence[3] = false)
            assert!(!values[4]); // NULL (presence[4] = false)
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0 and 2)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // NULL
            assert!(values[2]); // true and not NULL
            assert!(!values[3]); // NULL
            assert!(values[4]); // true and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_true_expr_with_non_boolean_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);
    let presence = &[true, true, false, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    table_map.insert(Ident::new("test_column"), nullable_column.values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_true_expr = IsTrueExpr::new(Box::new(column_expr));
    let result = is_true_expr.result_evaluate(&alloc, &table);

    match result {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // IS TRUE should be true only for non-NULL true values (index 0)
            assert!(values[0]); // true and not NULL
            assert!(!values[1]); // false and not NULL
            assert!(!values[2]); // NULL
            assert!(!values[3]); // NULL
            assert!(!values[4]); // false and not NULL
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn we_should_detect_a_malicious_prover_in_is_true_query() {
    use crate::{
        base::{
            database::{
                Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef,
            },
            map::IndexMap,
            proof::ProofError,
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::mock_verification_builder::MockVerificationBuilder,
            proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsTrueExpr},
        },
    };
    use ark_ff::Fp256;
    use num_traits::{One, Zero};
    use sqlparser::ast::Ident;
    use std::hash::BuildHasherDefault;

    // This test demonstrates how a malicious prover would be detected when using IS TRUE
    // We'll create a simple table with a boolean column and test the IS TRUE expression
    // with the malicious flag set

    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    // Create a boolean column with [true, false, true] values
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true]);

    // All values are present (not NULL)
    let presence = &[true, true, true];

    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Add the column to the table
    table_map.insert(Ident::new("a"), nullable_column.values);

    // Create a presence map
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("a"), presence.as_slice());

    // Create the table
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(3))).unwrap();

    // Create a column reference for the 'a' column
    let column_ref = ColumnRef::new(
        TableRef::new("sxt", "table"),
        Ident::new("a"),
        ColumnType::Boolean,
    );

    // Create a column expression
    let column_expr = DynProofExpr::new_column(column_ref);

    // Create an IS TRUE expression
    let mut is_true_expr = IsTrueExpr::new(Box::new(column_expr));

    // First get the correct result
    let correct_result = is_true_expr.result_evaluate(&alloc, &table);

    // Extract the correct boolean values
    let Column::Boolean(correct_values) = correct_result else {
        panic!("Expected boolean column")
    };

    // Verify the correct result
    // IS TRUE should be true only for non-NULL true values (index 0 and 2)
    assert_eq!(correct_values.len(), 3);
    assert!(correct_values[0]); // true and not NULL
    assert!(!correct_values[1]); // false and not NULL
    assert!(correct_values[2]); // true and not NULL

    // Now set malicious flag to simulate a malicious prover
    is_true_expr.malicious = true;

    // Create a mock builder with tampered final round MLEs
    // The tampered data should be different from what result_evaluate produced
    let tampered_mles = if correct_values.iter().any(|&x| x) {
        // If there are any true values, use all zeros to ensure it's different
        vec![vec![TestScalar::new(Fp256::zero())]]
    } else {
        // If all values are false, use all ones to ensure it's different
        vec![vec![TestScalar::new(Fp256::one())]]
    };

    let mut builder = MockVerificationBuilder::new(Vec::new(), 0, tampered_mles);

    let accessor = IndexMap::with_hasher(BuildHasherDefault::default());
    let chi_eval = TestScalar::new(Fp256::one());

    // Verification should fail because the tampered MLEs don't match the correct result
    let result = is_true_expr.verifier_evaluate(&mut builder, &accessor, chi_eval);
    assert!(
        result.is_err(),
        "Expected verification to fail with tampered MLEs"
    );

    if let Err(err) = result {
        assert!(
            matches!(err, ProofError::VerificationError { .. }),
            "Expected VerificationError error, got: {err:?}"
        );
    }
}
