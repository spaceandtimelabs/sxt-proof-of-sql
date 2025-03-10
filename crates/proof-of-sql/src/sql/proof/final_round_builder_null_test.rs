use super::FinalRoundBuilder;
use crate::base::{
    database::{Column, NullableColumn},
    scalar::test_scalar::TestScalar,
};
use alloc::collections::VecDeque;
use bumpalo::Bump;

#[test]
fn test_record_is_null_check_with_nulls() {
    // Create a test table with a nullable column with nulls
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence = &[true, false, true, false, true]; // Some values are NULL

    // Create a nullable column
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS NULL check
    builder.record_is_null_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (the presence column)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
fn test_record_is_null_check_without_nulls() {
    // Create a test table with a non-nullable column
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);

    // Create a nullable column without nulls
    let nullable_column = NullableColumn {
        values: column_values,
        presence: None,
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS NULL check
    builder.record_is_null_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (the presence column)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
fn test_record_is_not_null_check_with_nulls() {
    // Create a test table with a nullable column with nulls
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
    let presence = &[true, false, true, false, true]; // Some values are NULL

    // Create a nullable column
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS NOT NULL check
    builder.record_is_not_null_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (the presence column)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
fn test_record_is_not_null_check_without_nulls() {
    // Create a test table with a non-nullable column
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);

    // Create a nullable column without nulls
    let nullable_column = NullableColumn {
        values: column_values,
        presence: None,
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS NOT NULL check
    builder.record_is_not_null_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (constant true)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
fn test_record_is_true_check_with_nulls() {
    // Create a test table with a nullable column with nulls
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
    let presence = &[true, false, true, false, true]; // Some values are NULL

    // Create a nullable column
    let nullable_column = NullableColumn {
        values: column_values,
        presence: Some(presence),
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS TRUE check
    builder.record_is_true_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (the presence column)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
fn test_record_is_true_check_without_nulls() {
    // Create a test table with a non-nullable column
    let column_values: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, false]);

    // Create a nullable column without nulls
    let nullable_column = NullableColumn {
        values: column_values,
        presence: None,
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // Before the operation, there should be no intermediate MLEs
    assert_eq!(builder.pcs_proof_mles().len(), 0);

    // Record the IS TRUE check
    builder.record_is_true_check(&nullable_column, &alloc);

    // After the operation, there should be one intermediate MLE (the presence column)
    assert_eq!(builder.pcs_proof_mles().len(), 1);
}

#[test]
#[should_panic(expected = "IS TRUE can only be applied to boolean expressions")]
fn test_record_is_true_check_with_non_boolean_column() {
    // Create a test table with a non-boolean column
    let column_values: Column<'_, TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);

    // Create a nullable column
    let nullable_column = NullableColumn {
        values: column_values,
        presence: None,
    };

    // Create the allocator first
    let alloc = Bump::new();

    // Create a FinalRoundBuilder
    let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

    // This should panic because IS TRUE can only be applied to boolean expressions
    builder.record_is_true_check(&nullable_column, &alloc);
}
