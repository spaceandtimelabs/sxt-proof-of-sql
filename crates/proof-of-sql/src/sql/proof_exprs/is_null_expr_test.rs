use crate::{
    base::{
        bit::BitDistribution,
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table, TableOptions, TableRef},
        map::IndexMap,
        proof::ProofSizeMismatch,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::{proof_expr::ProofExpr, DynProofExpr, IsNullExpr},
    },
};
use alloc::{boxed::Box, collections::VecDeque};
use bumpalo::Bump;
use sqlparser::ast::Ident;
use std::hash::BuildHasherDefault;

#[test]
fn test_is_null_expr() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    // In our implementation, presence[i] = true means NOT NULL
    let presence = &[true, false, true, false, true];
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Insert the column values into the table map
    table_map.insert(Ident::new("test_column"), nullable_column.values);

    // Create a presence map to properly handle NULL values
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    // Create a ColumnRef directly instead of trying to convert from Ident
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));
    let result = is_null_expr.result_evaluate(&alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            // presence[i] = true means NOT NULL, so IS NULL should return false for those values
            assert!(!values[0]); // presence[0] = true -> IS NULL = false
            assert!(values[1]); // presence[1] = false -> IS NULL = true
            assert!(!values[2]); // presence[2] = true -> IS NULL = false
            assert!(values[3]); // presence[3] = false -> IS NULL = true
            assert!(!values[4]); // presence[4] = true -> IS NULL = false
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_non_nullable() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    table_map.insert(Ident::new("test_column"), column_values);

    let table = Table::try_new(table_map).unwrap();
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));
    let result = is_null_expr.result_evaluate(&alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            for &value in values {
                assert!(!value);
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_prover_evaluate() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    table_map.insert(Ident::new("test_column"), column_values);

    let table = Table::try_new(table_map).unwrap();
    let mut final_round_builder = FinalRoundBuilder::new(5, VecDeque::new());
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));
    let result = is_null_expr.prover_evaluate(&mut final_round_builder, &alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            for &value in values {
                assert!(!value);
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_verifier_evaluate() {
    let mut mock_builder = MockVerificationBuilder::new();
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Boolean,
    );

    let mut accessor = IndexMap::with_hasher(BuildHasherDefault::default());
    accessor.insert(column_ref.clone(), TestScalar::from(0));

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));

    mock_builder.add_mle_evaluation(TestScalar::from(0));
    mock_builder.add_mle_evaluation(TestScalar::from(0));
    mock_builder.add_mle_evaluation(TestScalar::from(1));

    let chi_eval = TestScalar::from(2);
    let result = is_null_expr.verifier_evaluate(&mut mock_builder, &accessor, chi_eval);
    match &result {
        Ok(value) => {
            assert_eq!((*value).0, TestScalar::from(1));
            assert!(mock_builder.produced_sumcheck);
        }
        Err(err) => {
            panic!("Test failed with error: {err:?}");
        }
    }
}

#[test]
fn test_is_null_expr_with_mixed_columns() {
    // This test ensures line 64 is covered by having a mix of nullable and non-nullable columns
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    // Add a nullable column
    let column_values1: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence1 = &[true, false, true, false, true];
    table_map.insert(Ident::new("nullable_column"), column_values1);

    // Add a non-nullable column
    let column_values2: Column<'_, TestScalar> = Column::Int(&[10, 20, 30, 40, 50]);
    table_map.insert(Ident::new("non_nullable_column"), column_values2);

    // Create presence map for the nullable column
    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("nullable_column"), presence1.as_slice());

    // Create the table with both column values and presence information
    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    // Create a ColumnRef for the nullable column
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("nullable_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));

    // This will exercise line 64 as has_nullable_column will be true
    let result = is_null_expr.result_evaluate(&alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            assert!(!values[0]);
            assert!(values[1]);
            assert!(!values[2]);
            assert!(values[3]);
            assert!(!values[4]);
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_prover_evaluate_with_nullable_column() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence = &[true, false, true, false, true];
    table_map.insert(Ident::new("test_column"), column_values);

    let mut presence_map = IndexMap::with_hasher(BuildHasherDefault::default());
    presence_map.insert(Ident::new("test_column"), presence.as_slice());

    let table =
        Table::try_new_with_presence(table_map, presence_map, TableOptions::new(Some(5))).unwrap();

    let mut final_round_builder = FinalRoundBuilder::new(5, VecDeque::new());
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));
    let result = is_null_expr.prover_evaluate(&mut final_round_builder, &alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            assert!(!values[0]);
            assert!(values[1]);
            assert!(!values[2]);
            assert!(values[3]);
            assert!(!values[4]);
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_verifier_evaluate_non_boolean() {
    let mut mock_builder = MockVerificationBuilder::new();
    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("test_column"),
        ColumnType::Int,
    );

    let mut accessor = IndexMap::with_hasher(BuildHasherDefault::default());
    accessor.insert(column_ref.clone(), TestScalar::from(42));

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));

    mock_builder.add_mle_evaluation(TestScalar::from(0));
    mock_builder.add_mle_evaluation(TestScalar::from(42));
    mock_builder.add_mle_evaluation(TestScalar::from(1));

    let chi_eval = TestScalar::from(2);
    let result = is_null_expr.verifier_evaluate(&mut mock_builder, &accessor, chi_eval);

    match &result {
        Ok(value) => {
            assert_eq!((*value).0, TestScalar::from(1));
            assert!(!mock_builder.produced_sumcheck);
        }
        Err(err) => {
            panic!("Test failed with error: {err:?}");
        }
    }
}

#[test]
fn test_is_null_expr_no_nullable_columns() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    table_map.insert(Ident::new("non_nullable_column"), column_values);

    let table = Table::try_new(table_map).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("non_nullable_column"),
        ColumnType::Int,
    );

    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));

    let result = is_null_expr.result_evaluate(&alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            for &value in values {
                assert!(!value);
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

#[test]
fn test_is_null_expr_prover_evaluate_no_nullable_columns() {
    let alloc = Bump::new();
    let mut table_map = IndexMap::with_hasher(BuildHasherDefault::default());

    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    table_map.insert(Ident::new("non_nullable_column"), column_values);

    let table = Table::try_new(table_map).unwrap();

    let column_ref = ColumnRef::new(
        TableRef::new("", "test"),
        Ident::new("non_nullable_column"),
        ColumnType::Int,
    );

    let mut final_round_builder = FinalRoundBuilder::new(5, VecDeque::new());
    let column_expr = DynProofExpr::new_column(column_ref);
    let is_null_expr = IsNullExpr::new(Box::new(column_expr));

    let result = is_null_expr.prover_evaluate(&mut final_round_builder, &alloc, &table);

    match result.values {
        Column::Boolean(values) => {
            assert_eq!(values.len(), 5);
            for &value in values {
                assert!(!value);
            }
        }
        _ => panic!("Expected boolean column"),
    }
}

struct MockVerificationBuilder {
    mle_evaluations: Vec<TestScalar>,
    current_index: usize,
    produced_sumcheck: bool,
}

impl MockVerificationBuilder {
    fn new() -> Self {
        Self {
            mle_evaluations: Vec::new(),
            current_index: 0,
            produced_sumcheck: false,
        }
    }

    fn add_mle_evaluation(&mut self, eval: TestScalar) {
        self.mle_evaluations.push(eval);
    }
}

impl VerificationBuilder<TestScalar> for MockVerificationBuilder {
    fn try_consume_chi_evaluation(&mut self) -> Result<TestScalar, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<TestScalar, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<TestScalar, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<TestScalar, ProofSizeMismatch> {
        if self.current_index < self.mle_evaluations.len() {
            let result = self.mle_evaluations[self.current_index];
            self.current_index += 1;
            Ok(result)
        } else {
            Err(ProofSizeMismatch::TooFewMLEEvaluations)
        }
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<TestScalar>, ProofSizeMismatch> {
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(self.try_consume_final_round_mle_evaluation()?);
        }
        Ok(result)
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        _typ: SumcheckSubpolynomialType,
        _eval: TestScalar,
        _degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        self.produced_sumcheck = true;
        Ok(())
    }

    fn try_consume_post_result_challenge(&mut self) -> Result<TestScalar, ProofSizeMismatch> {
        unimplemented!("No tests currently use this function")
    }

    fn singleton_chi_evaluation(&self) -> TestScalar {
        unimplemented!("No tests currently use this function")
    }

    fn rho_256_evaluation(&self) -> Option<TestScalar> {
        unimplemented!("No tests currently use this function")
    }
}
