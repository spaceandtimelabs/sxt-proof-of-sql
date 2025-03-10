use crate::{
    base::{
        database::{group_by_util::*, Column},
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use bumpalo::Bump;

#[test]
#[should_panic(expected = "MAX can not be applied to varbinary")]
fn we_cannot_apply_max_to_varbinary() {
    let col: Column<'_, TestScalar> = Column::VarBinary((&[], &[]));
    let indexes = &[];
    let counts = &[];
    let alloc = bumpalo::Bump::new();
    let _ = max_aggregate_column_by_index_counts(&alloc, &col, counts, indexes);
}

#[test]
#[should_panic(expected = "MIN can not be applied to varbinary")]
fn we_cannot_apply_min_to_varbinary() {
    let col: Column<'_, TestScalar> = Column::VarBinary((&[], &[]));
    let indexes = &[];
    let counts = &[];
    let alloc = bumpalo::Bump::new();
    let _ = min_aggregate_column_by_index_counts(&alloc, &col, counts, indexes);
}

#[test]
#[should_panic(expected = "MAX can not be applied to varchar")]
fn we_cannot_apply_max_to_varchar() {
    let col: Column<'_, TestScalar> = Column::VarChar((&[], &[]));
    let indexes = &[];
    let counts = &[];
    let alloc = bumpalo::Bump::new();
    let _ = max_aggregate_column_by_index_counts(&alloc, &col, counts, indexes);
}

#[test]
#[should_panic(expected = "MIN can not be applied to varchar")]
fn we_cannot_apply_min_to_varchar() {
    let col: Column<'_, TestScalar> = Column::VarChar((&[], &[]));
    let indexes = &[];
    let counts = &[];
    let alloc = bumpalo::Bump::new();
    let _ = min_aggregate_column_by_index_counts(&alloc, &col, counts, indexes);
}

#[test]
fn we_can_aggregate_empty_columns() {
    let column_a = Column::BigInt::<TestScalar>(&[]);
    let column_b = Column::VarChar((&[], &[]));
    let column_c = Column::Int128(&[]);
    let column_d = Column::Scalar(&[]);
    let group_by = &[column_a, column_b];
    let sum_columns = &[column_c, column_d];
    let selection = &[];
    let alloc = Bump::new();
    let aggregate_result = aggregate_columns(&alloc, group_by, sum_columns, &[], &[], selection)
        .expect("Aggregation should succeed");
    assert_eq!(
        aggregate_result.group_by_columns,
        vec![Column::BigInt(&[]), Column::VarChar((&[], &[]))]
    );
    assert_eq!(aggregate_result.sum_columns, vec![&[], &[]]);
    assert_eq!(aggregate_result.count_column, &[0i64; 0]);
}

#[test]
fn we_can_aggregate_columns_with_empty_group_by_and_no_rows_selected() {
    let slice_c = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111];
    let slice_d = &[200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211];
    let selection = &[false; 12];
    let scals_d: Vec<TestScalar> = slice_d.iter().map(core::convert::Into::into).collect();
    let column_c = Column::Int128(slice_c);
    let column_d = Column::Scalar(&scals_d);
    let group_by = &[];
    let sum_columns = &[column_c, column_d];
    let max_columns = &[column_c, column_d];
    let min_columns = &[column_c, column_d];
    let alloc = Bump::new();
    let aggregate_result = aggregate_columns(
        &alloc,
        group_by,
        sum_columns,
        min_columns,
        max_columns,
        selection,
    )
    .expect("Aggregation should succeed");
    let expected_group_by_result = &[];
    let expected_sum_result = &[&[], &[]];
    let expected_max_result = &[&[], &[]];
    let expected_min_result = &[&[], &[]];
    let expected_count_result: &[i64] = &[];
    assert_eq!(aggregate_result.group_by_columns, expected_group_by_result);
    assert_eq!(aggregate_result.sum_columns, expected_sum_result);
    assert_eq!(aggregate_result.count_column, expected_count_result);
    assert_eq!(aggregate_result.max_columns, expected_max_result);
    assert_eq!(aggregate_result.min_columns, expected_min_result);
}

#[test]
fn we_can_aggregate_columns_with_empty_group_by() {
    let slice_c = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111];
    let slice_d = &[200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211];
    let selection = &[
        false, true, true, true, true, true, true, true, true, true, true, true,
    ];
    let scals_d: Vec<TestScalar> = slice_d.iter().map(core::convert::Into::into).collect();
    let column_c = Column::Int128(slice_c);
    let column_d = Column::Scalar(&scals_d);
    let group_by = &[];
    let sum_columns = &[column_c, column_d];
    let max_columns = &[column_c, column_d];
    let min_columns = &[column_c, column_d];
    let alloc = Bump::new();
    let aggregate_result = aggregate_columns(
        &alloc,
        group_by,
        sum_columns,
        min_columns,
        max_columns,
        selection,
    )
    .expect("Aggregation should succeed");
    let expected_group_by_result = &[];
    let expected_sum_result = &[
        &[TestScalar::from(
            101 + 102 + 103 + 104 + 105 + 106 + 107 + 108 + 109 + 110 + 111,
        )],
        &[TestScalar::from(
            201 + 202 + 203 + 204 + 205 + 206 + 207 + 208 + 209 + 210 + 211,
        )],
    ];
    let expected_max_result = &[
        &[Some(TestScalar::from(111))],
        &[Some(TestScalar::from(211))],
    ];
    let expected_min_result = &[
        &[Some(TestScalar::from(101))],
        &[Some(TestScalar::from(201))],
    ];
    let expected_count_result = &[11];
    assert_eq!(aggregate_result.group_by_columns, expected_group_by_result);
    assert_eq!(aggregate_result.sum_columns, expected_sum_result);
    assert_eq!(aggregate_result.count_column, expected_count_result);
    assert_eq!(aggregate_result.max_columns, expected_max_result);
    assert_eq!(aggregate_result.min_columns, expected_min_result);
}

#[allow(clippy::too_many_lines)]
#[test]
fn we_can_aggregate_columns() {
    let slice_a = &[3, 3, 3, 2, 2, 1, 1, 2, 2, 3, 3, 3];
    let slice_b = &[
        "Cat", "Cat", "Dog", "Cat", "Dog", "Cat", "Dog", "Cat", "Dog", "Cat", "Dog", "Cat",
    ];
    let slice_c = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111];
    let slice_d = &[200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211];
    let selection = &[
        false, true, true, true, true, true, true, true, true, true, true, true,
    ];
    let scals_b: Vec<TestScalar> = slice_b.iter().map(core::convert::Into::into).collect();
    let scals_d: Vec<TestScalar> = slice_d.iter().map(core::convert::Into::into).collect();
    let column_a = Column::BigInt(slice_a);
    let column_b = Column::VarChar((slice_b, &scals_b));
    let column_c = Column::Int128(slice_c);
    let column_d = Column::Scalar(&scals_d);
    let group_by = &[column_a, column_b];
    let sum_columns = &[column_c, column_d];
    let max_columns = &[column_c, column_d];
    let min_columns = &[column_c, column_d];
    let alloc = Bump::new();
    let aggregate_result = aggregate_columns(
        &alloc,
        group_by,
        sum_columns,
        min_columns,
        max_columns,
        selection,
    )
    .expect("Aggregation should succeed");
    let scals_res = [
        TestScalar::from("Cat"),
        TestScalar::from("Dog"),
        TestScalar::from("Cat"),
        TestScalar::from("Dog"),
        TestScalar::from("Cat"),
        TestScalar::from("Dog"),
    ];
    let expected_group_by_result = &[
        Column::BigInt(&[1, 1, 2, 2, 3, 3]),
        Column::VarChar((&["Cat", "Dog", "Cat", "Dog", "Cat", "Dog"], &scals_res)),
    ];
    let expected_sum_result = &[
        &[
            TestScalar::from(105),
            TestScalar::from(106),
            TestScalar::from(103 + 107),
            TestScalar::from(104 + 108),
            TestScalar::from(101 + 109 + 111),
            TestScalar::from(102 + 110),
        ],
        &[
            TestScalar::from(205),
            TestScalar::from(206),
            TestScalar::from(203 + 207),
            TestScalar::from(204 + 208),
            TestScalar::from(201 + 209 + 211),
            TestScalar::from(202 + 210),
        ],
    ];
    let expected_max_result = &[
        &[
            Some(TestScalar::from(105)),
            Some(TestScalar::from(106)),
            Some(TestScalar::from(107)),
            Some(TestScalar::from(108)),
            Some(TestScalar::from(111)),
            Some(TestScalar::from(110)),
        ],
        &[
            Some(TestScalar::from(205)),
            Some(TestScalar::from(206)),
            Some(TestScalar::from(207)),
            Some(TestScalar::from(208)),
            Some(TestScalar::from(211)),
            Some(TestScalar::from(210)),
        ],
    ];
    let expected_min_result = &[
        &[
            Some(TestScalar::from(105)),
            Some(TestScalar::from(106)),
            Some(TestScalar::from(103)),
            Some(TestScalar::from(104)),
            Some(TestScalar::from(101)),
            Some(TestScalar::from(102)),
        ],
        &[
            Some(TestScalar::from(205)),
            Some(TestScalar::from(206)),
            Some(TestScalar::from(203)),
            Some(TestScalar::from(204)),
            Some(TestScalar::from(201)),
            Some(TestScalar::from(202)),
        ],
    ];
    let expected_count_result = &[1, 1, 2, 2, 3, 2];
    assert_eq!(aggregate_result.group_by_columns, expected_group_by_result);
    assert_eq!(aggregate_result.sum_columns, expected_sum_result);
    assert_eq!(aggregate_result.count_column, expected_count_result);
    assert_eq!(aggregate_result.max_columns, expected_max_result);
    assert_eq!(aggregate_result.min_columns, expected_min_result);
}

// SUM slices
#[test]
fn we_can_sum_aggregate_slice_by_counts_for_empty_slice() {
    let slice_a: &[i64; 0] = &[];
    let indexes = &[];
    let counts = &[];
    let expected: &[DoryScalar; 0] = &[];
    let alloc = Bump::new();
    let result: &[DoryScalar] =
        sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_slice_by_counts_with_empty_result() {
    let slice_a = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
    let indexes = &[];
    let counts = &[];
    let expected: &[DoryScalar; 0] = &[];
    let alloc = Bump::new();
    let result: &[DoryScalar] =
        sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_slice_by_counts_with_all_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[];
    let counts = &[0, 0, 0];
    let expected = &[TestScalar::from(0); 3];
    let alloc = Bump::new();
    let result: &[TestScalar] =
        sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_slice_by_counts_with_some_empty_group() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 4];
    let counts = &[3, 4, 0];
    let expected = &[
        TestScalar::from(112 + 111 + 101),
        TestScalar::from(110 + 102 + 103 + 104),
        TestScalar::from(0),
    ];
    let alloc = Bump::new();
    let result: &[TestScalar] =
        sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_slice_by_counts_without_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4];
    let expected = &[
        TestScalar::from(112 + 111 + 101),
        TestScalar::from(110 + 102 + 103),
        TestScalar::from(106 + 114 + 113 + 109),
    ];
    let alloc = Bump::new();
    let result: &[TestScalar] =
        sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_columns_by_counts_for_empty_column() {
    let slice_a: &[i64; 0] = &[];
    let column_a = Column::BigInt::<DoryScalar>(slice_a);
    let indexes = &[];
    let counts = &[];
    let expected: &[DoryScalar; 0] = &[];
    let alloc = Bump::new();
    let result: &[DoryScalar] =
        sum_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_sum_aggregate_columns_by_counts() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let slice_b = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let slice_c = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let scals_c: Vec<TestScalar> = slice_c.iter().map(core::convert::Into::into).collect();
    let column_a = Column::BigInt::<TestScalar>(slice_a);
    let columns_b = Column::Int128::<TestScalar>(slice_b);
    let columns_c = Column::Scalar(&scals_c);
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4];
    let expected = &[
        TestScalar::from(112 + 111 + 101),
        TestScalar::from(110 + 102 + 103),
        TestScalar::from(106 + 114 + 113 + 109),
    ];
    let alloc = Bump::new();
    let result = sum_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
    let result = sum_aggregate_column_by_index_counts(&alloc, &columns_b, counts, indexes);
    assert_eq!(result, expected);
    let result = sum_aggregate_column_by_index_counts(&alloc, &columns_c, counts, indexes);
    assert_eq!(result, expected);
}

// MAX slices
#[test]
fn we_can_max_aggregate_slice_by_counts_for_empty_slice() {
    let slice_a: &[i64; 0] = &[];
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_slice_by_counts_with_empty_result() {
    let slice_a = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_slice_by_counts_with_all_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[];
    let counts = &[0, 0, 0];
    let expected = &[None; 3];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_slice_by_counts_with_some_empty_group() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 4];
    let counts = &[3, 4, 0];
    let expected = &[
        Some(TestScalar::from(112)),
        Some(TestScalar::from(110)),
        None,
    ];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_slice_by_counts_without_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4];
    let expected = &[
        Some(TestScalar::from(112)),
        Some(TestScalar::from(110)),
        Some(TestScalar::from(114)),
    ];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        max_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_columns_by_counts_for_empty_column() {
    let slice_a: &[i64; 0] = &[];
    let column_a = Column::BigInt::<DoryScalar>(slice_a);
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        max_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_max_aggregate_columns_by_counts() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let slice_b = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let slice_c = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let scals_c: Vec<TestScalar> = slice_c.iter().map(core::convert::Into::into).collect();
    let column_a = Column::BigInt::<TestScalar>(slice_a);
    let columns_b = Column::Int128::<TestScalar>(slice_b);
    let columns_c = Column::Scalar(&scals_c);
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4, 0];
    let expected = &[
        Some(TestScalar::from(112)),
        Some(TestScalar::from(110)),
        Some(TestScalar::from(114)),
        None,
    ];
    let alloc = Bump::new();
    let result = max_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
    let result = max_aggregate_column_by_index_counts(&alloc, &columns_b, counts, indexes);
    assert_eq!(result, expected);
    let result = max_aggregate_column_by_index_counts(&alloc, &columns_c, counts, indexes);
    assert_eq!(result, expected);
}

// MIN slices
#[test]
fn we_can_min_aggregate_slice_by_counts_for_empty_slice() {
    let slice_a: &[i64; 0] = &[];
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_slice_by_counts_with_empty_result() {
    let slice_a = &[100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_slice_by_counts_with_all_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[];
    let counts = &[0, 0, 0];
    let expected = &[None; 3];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_slice_by_counts_with_some_empty_group() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 4];
    let counts = &[3, 4, 0];
    let expected = &[
        Some(TestScalar::from(101)),
        Some(TestScalar::from(102)),
        None,
    ];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_slice_by_counts_without_empty_groups() {
    let slice_a = &[
        100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    ];
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4];
    let expected = &[
        Some(TestScalar::from(101)),
        Some(TestScalar::from(102)),
        Some(TestScalar::from(106)),
    ];
    let alloc = Bump::new();
    let result: &[Option<TestScalar>] =
        min_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_columns_by_counts_for_empty_column() {
    let slice_a: &[i64; 0] = &[];
    let column_a = Column::BigInt::<DoryScalar>(slice_a);
    let indexes = &[];
    let counts = &[];
    let expected: &[Option<DoryScalar>; 0] = &[];
    let alloc = Bump::new();
    let result: &[Option<DoryScalar>] =
        min_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_min_aggregate_columns_by_counts() {
    let slice_a = &[
        100, -101, 102, -103, 104, -105, 106, -107, 108, -109, 110, -111, 112, -113, 114, -115,
    ];
    let slice_b = &[
        100, -101, 102, -103, 104, -105, 106, -107, 108, -109, 110, -111, 112, -113, 114, -115,
    ];
    let slice_c = &[
        100, -101, 102, -103, 104, -105, 106, -107, 108, -109, 110, -111, 112, -113, 114, -115,
    ];
    let scals_c: Vec<TestScalar> = slice_c.iter().map(core::convert::Into::into).collect();
    let column_a = Column::BigInt::<TestScalar>(slice_a);
    let columns_b = Column::Int128::<TestScalar>(slice_b);
    let columns_c = Column::Scalar(&scals_c);
    let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
    let counts = &[3, 3, 4, 0];
    let expected = &[
        Some(TestScalar::from(-111)),
        Some(TestScalar::from(-103)),
        Some(TestScalar::from(-113)),
        None,
    ];
    let alloc = Bump::new();
    let result = min_aggregate_column_by_index_counts(&alloc, &column_a, counts, indexes);
    assert_eq!(result, expected);
    let result = min_aggregate_column_by_index_counts(&alloc, &columns_b, counts, indexes);
    assert_eq!(result, expected);
    let result = min_aggregate_column_by_index_counts(&alloc, &columns_c, counts, indexes);
    assert_eq!(result, expected);
}

#[test]
fn we_can_aggregate_columns_with_varbinary_in_group_by() {
    let raw_bytes = [
        b"foo".as_ref(),
        b"bar".as_ref(),
        b"bar".as_ref(),
        b"baz".as_ref(),
    ];
    let scalars: Vec<TestScalar> = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect();

    let col_varbinary = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));
    let col_bigint = Column::BigInt(&[10, 20, 30, 40]);
    let binding = [
        TestScalar::from(100),
        TestScalar::from(200),
        TestScalar::from(300),
        TestScalar::from(400),
    ];
    let col_scalar = Column::Scalar(&binding);

    let selection = &[true, true, true, true];

    let alloc = Bump::new();
    let group_by = &[col_varbinary];
    let sum_columns = &[col_bigint, col_scalar];
    let min_columns = &[];
    let max_columns = &[];

    let aggregate_result = aggregate_columns(
        &alloc,
        group_by,
        sum_columns,
        min_columns,
        max_columns,
        selection,
    )
    .expect("Aggregation should succeed");

    let expected_bytes = [b"bar".as_ref(), b"baz".as_ref(), b"foo".as_ref()];
    let expected_bytes_scalars: Vec<TestScalar> = expected_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect();

    let expected_group_by = vec![Column::VarBinary((
        expected_bytes.as_slice(),
        expected_bytes_scalars.as_slice(),
    ))];

    let expected_sum_bigint = [
        TestScalar::from(50),
        TestScalar::from(40),
        TestScalar::from(10),
    ];

    let expected_sum_scalar = [
        TestScalar::from(500),
        TestScalar::from(400),
        TestScalar::from(100),
    ];

    let expected_sum = vec![&expected_sum_bigint[..], &expected_sum_scalar[..]];

    let expected_count = &[2, 1, 1];

    assert_eq!(aggregate_result.group_by_columns, expected_group_by);
    assert_eq!(aggregate_result.sum_columns, expected_sum);
    assert_eq!(aggregate_result.count_column, expected_count);
    assert!(aggregate_result.max_columns.is_empty());
    assert!(aggregate_result.min_columns.is_empty());
}
