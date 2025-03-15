use crate::base::{
    database::{filter_util::*, Column},
    math::decimal::Precision,
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

#[test]
fn we_can_filter_columns() {
    let selection = vec![true, false, true, false, true];
    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), 0, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[1, 3, 5]),
            Column::Int128(&[1, 3, 5]),
            Column::VarChar((&["1", "3", "5"], &["1".into(), "3".into(), "5".into()])),
            Column::Scalar(&[1.into(), 3.into(), 5.into()]),
            Column::Decimal75(
                Precision::new(75).unwrap(),
                0,
                &[1.into(), 3.into(), 5.into()]
            )
        ]
    );
}
#[test]
fn we_can_filter_columns_with_empty_result() {
    let selection = vec![false, false, false, false, false];
    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}
#[test]
fn we_can_filter_empty_columns() {
    let selection = vec![];
    let columns = vec![
        Column::<TestScalar>::BigInt(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}

#[test]
fn we_can_filter_nullable_columns_with_null_operands() {
    use crate::base::database::NullableColumn;
    
    // Create test data with a mixture of null and non-null values
    let selection = vec![false, true, false, true, false]; // Result from WHERE clause evaluation
    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let null_presence = vec![true, true, false, true, false]; // true = value is present, false = NULL
    
    // Create columns with both values and null indicators
    let int_column = Column::BigInt(&[1, 2, 3, 4, 5]);
    let varchar_column = Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars));
    
    // Create nullable columns
    let nullable_int = NullableColumn::with_presence(int_column, Some(&null_presence)).unwrap();
    let nullable_varchar = NullableColumn::with_presence(varchar_column, Some(&null_presence)).unwrap();
    
    // Create a non-nullable column
    let non_nullable_column = NullableColumn::new(Column::Int128(&[1, 2, 3, 4, 5]));
    
    // The columns to filter
    let columns = vec![
        nullable_int.values,
        nullable_varchar.values,
        non_nullable_column.values,
    ];
    
    // Allocator for temporary memory
    let alloc = Bump::new();
    
    // Perform the filtering - this simulates filtering rows where a WHERE clause condition is true
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    
    // Verify results
    assert_eq!(len, 2); // Only 2 rows match the selection criteria
    
    // Check filtered column data
    assert_eq!(result[0], Column::BigInt(&[2, 4]));
    assert_eq!(result[1], Column::VarChar((&["2", "4"], &["2".into(), "4".into()])));
    assert_eq!(result[2], Column::Int128(&[2, 4]));
    
    // Now let's check a more complex case simulating a WHERE clause involving NULL values
    // In SQL, conditions with NULL operands typically evaluate to false
    
    // Create a selection representing: a > 2 AND b IS NOT NULL
    // For a=[1, 2, 3, 4, 5] with NULL at positions 2 and 4
    // For this condition, only position 3 (value 4) should pass
    let complex_selection = vec![false, false, false, true, false];
    
    let (result2, len2) = filter_columns(&alloc, &columns, &complex_selection);
    
    // Verify results
    assert_eq!(len2, 1); // Only 1 row matches
    assert_eq!(result2[0], Column::BigInt(&[4]));
    assert_eq!(result2[1], Column::VarChar((&["4"], &["4".into()])));
    assert_eq!(result2[2], Column::Int128(&[4]));
}

#[test]
fn we_can_filter_complex_null_cases_with_multiple_where_clauses() {
    use crate::base::database::NullableColumn;
    
    // Create test data with 7 rows and a mix of NULL values
    // Values represent: id, value_a, name_b, score_c
    let ids = [1, 2, 3, 4, 5, 6, 7];
    let value_a = [10, 20, 30, 40, 50, 60, 70];
    let name_b_strs = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Frank", "Grace"];
    let name_b_scalars: Vec<TestScalar> = name_b_strs.iter().map(|s| (*s).into()).collect();
    
    // Store scores as i64 since TestScalar doesn't support direct float conversion
    let score_c_i64 = [5i64, 3i64, 4i64, 2i64, 3i64, 4i64, 3i64];
    let score_c_scalars: Vec<TestScalar> = score_c_i64.iter().map(|&s| s.into()).collect();
    
    // Create NULL presence indicators (true = value present, false = NULL)
    // - value_a: NULL at rows 2, 5
    // - name_b: NULL at rows 3, 6
    // - score_c: NULL at rows 1, 4, 7
    let value_a_presence = [true, false, true, true, false, true, true];
    let name_b_presence = [true, true, false, true, true, false, true]; 
    let score_c_presence = [false, true, true, false, true, true, false];
    
    // Create columns
    let id_column = Column::BigInt(&ids);
    let value_a_column = Column::Int(&value_a);
    let name_b_column = Column::VarChar((name_b_strs.as_slice(), name_b_scalars.as_slice()));
    let score_c_column = Column::Scalar(score_c_scalars.as_slice());
    
    // Create nullable columns
    let nullable_value_a = NullableColumn::with_presence(value_a_column, Some(&value_a_presence)).unwrap();
    let nullable_name_b = NullableColumn::with_presence(name_b_column, Some(&name_b_presence)).unwrap();
    let nullable_score_c = NullableColumn::with_presence(score_c_column, Some(&score_c_presence)).unwrap();
    
    // Non-nullable id column
    let id = NullableColumn::new(id_column);
    
    // The columns to filter
    let columns = vec![
        id.values,
        nullable_value_a.values,
        nullable_name_b.values,
        nullable_score_c.values,
    ];

    let alloc = Bump::new();
    
    // SCENARIO 1: Filter rows where value_a > 25
    // In SQL: SELECT * FROM table WHERE value_a > 25
    // Expected result: rows 3, 4, 6, 7 (where value_a is 30, 40, 60, 70)
    // But in row 5, value_a is NULL, so the comparison is false
    let filter1 = vec![false, false, true, true, false, true, true];
    
    let (result1, len1) = filter_columns(&alloc, &columns, &filter1);
    
    // Verify results
    assert_eq!(len1, 4);
    assert_eq!(result1[0], Column::BigInt(&[3, 4, 6, 7]));
    assert_eq!(result1[1], Column::Int(&[30, 40, 60, 70]));
    assert_eq!(
        result1[2], 
        Column::VarChar((
            &["Charlie", "Dave", "Frank", "Grace"], 
            &["Charlie".into(), "Dave".into(), "Frank".into(), "Grace".into()]
        ))
    );
    assert_eq!(
        result1[3], 
        Column::Scalar(&[4i64.into(), 2i64.into(), 4i64.into(), 3i64.into()])
    );
    
    // SCENARIO 2: Filter rows where name_b IS NULL
    // In SQL: SELECT * FROM table WHERE name_b IS NULL
    // Expected result: rows 3, 6 (where name_b is NULL)
    let filter2 = vec![false, false, true, false, false, true, false];
    
    let (result2, len2) = filter_columns(&alloc, &columns, &filter2);
    
    // Verify results
    assert_eq!(len2, 2);
    assert_eq!(result2[0], Column::BigInt(&[3, 6]));
    assert_eq!(result2[1], Column::Int(&[30, 60]));
    // The string values don't matter for NULL entries, but they're still represented in the column
    assert_eq!(
        result2[2], 
        Column::VarChar((
            &["Charlie", "Frank"], 
            &["Charlie".into(), "Frank".into()]
        ))
    );
    assert_eq!(
        result2[3], 
        Column::Scalar(&[4i64.into(), 4i64.into()])
    );
    
    // SCENARIO 3: Complex condition with OR and IS NOT NULL
    // In SQL: SELECT * FROM table WHERE value_a > 40 OR (score_c IS NOT NULL AND name_b = 'Bob')
    // Expected results: rows 2, 5, 6, 7 (where value_a > 40 OR (score_c IS NOT NULL AND name_b = 'Bob'))
    // Row 2: name_b = 'Bob' AND score_c IS NOT NULL
    // Rows 5, 6, 7: value_a > 40
    let filter3 = vec![false, true, false, false, true, true, true];
    
    let (result3, len3) = filter_columns(&alloc, &columns, &filter3);
    
    // Verify results
    assert_eq!(len3, 4);
    assert_eq!(result3[0], Column::BigInt(&[2, 5, 6, 7]));
    assert_eq!(result3[1], Column::Int(&[20, 50, 60, 70]));
    assert_eq!(
        result3[2], 
        Column::VarChar((
            &["Bob", "Eve", "Frank", "Grace"], 
            &["Bob".into(), "Eve".into(), "Frank".into(), "Grace".into()]
        ))
    );
    assert_eq!(
        result3[3], 
        Column::Scalar(&[3i64.into(), 3i64.into(), 4i64.into(), 3i64.into()])
    );
}

#[test]
fn we_can_filter_columns_with_varbinary() {
    let selection = vec![true, false, true, true, false];
    let raw_bytes = [b"foo".as_ref(), b"bar", b"baz", b"qux", b"quux"];
    let scalars: [TestScalar; 5] = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let columns = vec![
        Column::VarBinary((&raw_bytes, &scalars)),
        Column::BigInt(&[10, 20, 30, 40, 50]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    let filtered_bytes = [b"foo".as_ref(), b"baz", b"qux"];
    let filtered_scalars = filtered_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>();
    assert_eq!(
        result,
        vec![
            Column::VarBinary((filtered_bytes.as_slice(), filtered_scalars.as_slice())),
            Column::BigInt(&[10, 30, 40]),
        ]
    );
}
