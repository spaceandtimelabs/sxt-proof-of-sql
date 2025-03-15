use crate::{
    base::{
        database::{owned_table_utility::*, OwnedColumn, OwnedTable, OwnedTableError},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use sqlparser::ast::Ident;
#[test]
fn we_can_create_an_owned_table_with_no_columns() {
    let table = OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap();
    assert_eq!(table.num_columns(), 0);
}
#[test]
fn we_can_create_an_empty_owned_table() {
    let owned_table = owned_table::<DoryScalar>([
        bigint("bigint", [0; 0]),
        int128("decimal", [0; 0]),
        varchar("varchar", ["0"; 0]),
        scalar("scalar", [0; 0]),
        boolean("boolean", [true; 0]),
    ]);
    let mut table = IndexMap::default();
    table.insert(Ident::new("bigint"), OwnedColumn::BigInt(vec![]));
    table.insert(Ident::new("decimal"), OwnedColumn::Int128(vec![]));
    table.insert(Ident::new("varchar"), OwnedColumn::VarChar(vec![]));
    table.insert(Ident::new("scalar"), OwnedColumn::Scalar(vec![]));
    table.insert(Ident::new("boolean"), OwnedColumn::Boolean(vec![]));
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_can_create_an_owned_table_with_data() {
    let owned_table = owned_table([
        bigint("bigint", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
        int128("decimal", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
        varchar("varchar", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
        scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        boolean(
            "boolean",
            [true, false, true, false, true, false, true, false, true],
        ),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
        ),
    ]);
    let mut table = IndexMap::default();
    table.insert(
        Ident::new("time_stamp"),
        OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX].into(),
        ),
    );
    table.insert(
        Ident::new("bigint"),
        OwnedColumn::BigInt(vec![0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
    );
    table.insert(
        Ident::new("decimal"),
        OwnedColumn::Int128(vec![0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
    );
    table.insert(
        Ident::new("varchar"),
        OwnedColumn::VarChar(vec![
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
        ]),
    );
    table.insert(
        Ident::new("scalar"),
        OwnedColumn::Scalar(vec![
            DoryScalar::from(0),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            7.into(),
            8.into(),
        ]),
    );
    table.insert(
        Ident::new("boolean"),
        OwnedColumn::Boolean(vec![
            true, false, true, false, true, false, true, false, true,
        ]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let owned_table_a: OwnedTable<TestScalar> = owned_table([
        bigint("a", [0; 0]),
        int128("b", [0; 0]),
        varchar("c", ["0"; 0]),
        boolean("d", [false; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0; 0],
        ),
    ]);
    let owned_table_b: OwnedTable<TestScalar> = owned_table([
        boolean("d", [false; 0]),
        int128("b", [0; 0]),
        bigint("a", [0; 0]),
        varchar("c", ["0"; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0; 0],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let owned_table_a: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [0]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_625_072_400],
        ),
    ]);
    let owned_table_b: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [1]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_625_076_000],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_cannot_create_an_owned_table_with_differing_column_lengths() {
    assert!(matches!(
        OwnedTable::<TestScalar>::try_from_iter([
            ("a".into(), OwnedColumn::BigInt(vec![0])),
            ("b".into(), OwnedColumn::BigInt(vec![])),
        ]),
        Err(OwnedTableError::ColumnLengthMismatch)
    ));
}
#[test]
fn we_can_perform_null_operations_with_where_clause_in_three_valued_logic() {
    use crate::base::database::{
        owned_column::OwnedNullableColumn,
        owned_table_utility::{
            bigint, bigint_values, boolean, boolean_values, int, nullable_column_pair, owned_table_with_nulls,
            smallint, varchar_values,
        },
    };

    // Create a table with multiple columns and rows, with various NULL patterns
    // We'll create 10 rows with different NULL patterns across 6 different columns
    
    // Column A: BigInt with some NULL values
    let a_values = bigint_values::<DoryScalar>([10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
    let a_presence = Some(vec![true, true, false, true, false, true, true, false, true, true]);
    
    // Column B: Int with some NULL values (different pattern)
    let b_values = OwnedColumn::Int(vec![5, 15, 25, 35, 45, 55, 65, 75, 75, 95]);
    let b_presence = Some(vec![true, false, true, false, true, true, false, true, true, false]);
    
    // Column C: SmallInt with some NULL values (another pattern)
    let c_values = OwnedColumn::SmallInt(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let c_presence = Some(vec![false, true, true, true, false, false, true, true, false, true]);
    
    // Column D: VarChar with some NULL values
    let d_values = varchar_values::<DoryScalar>(["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);
    let d_presence = Some(vec![true, true, true, false, false, true, false, true, true, false]);
    
    // Column E: Boolean with some NULL values
    let e_values = boolean_values::<DoryScalar>([true, false, true, false, true, false, true, false, true, false]);
    let e_presence = Some(vec![true, false, false, true, true, false, true, true, false, true]);
    
    // Column F: BigInt with no NULL values
    let f_values = bigint_values::<DoryScalar>([100, 90, 80, 70, 60, 50, 40, 30, 20, 10]);
    
    // Create the table with all these columns
    let table = owned_table_with_nulls([
        nullable_column_pair("a", a_values.clone(), a_presence.clone()),
        nullable_column_pair("b", b_values.clone(), b_presence.clone()),
        nullable_column_pair("c", c_values.clone(), c_presence.clone()),
        nullable_column_pair("d", d_values.clone(), d_presence.clone()),
        nullable_column_pair("e", e_values.clone(), e_presence.clone()),
        nullable_column_pair("f", f_values.clone(), None),
    ]);
    
    // Now let's simulate various WHERE clause operations
    
    // 1. Simple equality: WHERE a = 50
    let a_col = OwnedNullableColumn::with_presence(a_values.clone(), a_presence.clone()).unwrap();
    let fifty = OwnedNullableColumn::new(bigint_values::<DoryScalar>([50; 10]));
    let a_eq_50 = a_col.element_wise_eq(&fifty).unwrap();
    
    // In SQL, only rows where the condition is TRUE (not NULL, not FALSE) are included
    // For a = 50, only row 5 has a=50, but it's NULL, so no rows should match
    let where_result_1: Vec<bool> = a_eq_50.presence
        .unwrap()
        .iter()
        .zip(match &a_eq_50.values {
            OwnedColumn::Boolean(values) => values.iter(),
            _ => panic!("Expected boolean column"),
        })
        .map(|(present, value)| *present && *value)
        .collect();
    
    // No rows should match a = 50 because row 5 has a=NULL
    assert_eq!(where_result_1, vec![false, false, false, false, false, false, false, false, false, false]);
    
    // 2. Complex condition: WHERE a > 50 AND b < 80
    let a_col = OwnedNullableColumn::with_presence(a_values.clone(), a_presence.clone()).unwrap();
    let fifty_again = OwnedNullableColumn::new(bigint_values::<DoryScalar>([50; 10]));
    let a_gt_50 = a_col.element_wise_gt(&fifty_again).unwrap();
    
    let b_col = OwnedNullableColumn::with_presence(b_values.clone(), b_presence.clone()).unwrap();
    let eighty = OwnedNullableColumn::new(OwnedColumn::Int(vec![80; 10]));
    let b_lt_80 = b_col.element_wise_lt(&eighty).unwrap();
    
    // AND operation in three-valued logic
    let and_result = a_gt_50.element_wise_and(&b_lt_80).unwrap();
    
    // Extract rows that satisfy the WHERE clause (condition is TRUE, not NULL or FALSE)
    let where_result_2: Vec<bool> = and_result.presence
        .unwrap()
        .iter()
        .zip(match &and_result.values {
            OwnedColumn::Boolean(values) => values.iter(),
            _ => panic!("Expected boolean column"),
        })
        .map(|(present, value)| *present && *value)
        .collect();
    
    // Rows 6, 9 should match (a > 50 AND b < 80)
    // Row 6: a=60, b=55 -> true AND true -> true
    // Row 9: a=90, b=75 -> true AND true -> true
    assert_eq!(where_result_2, vec![false, false, false, false, false, true, false, false, true, false]);
    
    // 3. OR with NULL: WHERE a < 30 OR c IS NULL
    let a_col = OwnedNullableColumn::with_presence(a_values.clone(), a_presence.clone()).unwrap();
    let thirty = OwnedNullableColumn::new(bigint_values::<DoryScalar>([30; 10]));
    let a_lt_30 = a_col.element_wise_lt(&thirty).unwrap();
    
    // For IS NULL, we need to check the presence vector directly
    let c_is_null: Vec<bool> = c_presence.as_ref()
        .unwrap()
        .iter()
        .map(|present| !present)
        .collect();
    
    // Convert c_is_null to a nullable column
    let c_is_null_col = OwnedNullableColumn::new(
        OwnedColumn::Boolean(c_is_null)
    );
    
    // OR operation
    let or_result = a_lt_30.element_wise_or(&c_is_null_col).unwrap();
    
    // Extract rows that satisfy the WHERE clause
    let where_result_3: Vec<bool> = or_result.presence
        .unwrap()
        .iter()
        .zip(match &or_result.values {
            OwnedColumn::Boolean(values) => values.iter(),
            _ => panic!("Expected boolean column"),
        })
        .map(|(present, value)| *present && *value)
        .collect();
    
    // Rows 1, 2, 5, 6, 9 should match (a < 30 OR c IS NULL)
    // Row 1: a=10, c=NULL -> true OR true -> true
    // Row 2: a=20, c=2 -> true OR false -> true
    // Row 5: a=NULL, c=NULL -> NULL OR true -> true
    // Row 6: a=60, c=NULL -> false OR true -> true
    // Row 9: a=90, c=NULL -> false OR true -> true
    assert_eq!(where_result_3, vec![true, true, false, false, true, true, false, false, true, false]);
    
    // 4. Complex condition with multiple operations: WHERE (a > b OR c < 5) AND e IS NOT NULL
    let a_col = OwnedNullableColumn::with_presence(a_values, a_presence).unwrap();
    let b_col = OwnedNullableColumn::with_presence(b_values, b_presence).unwrap();
    let a_gt_b = a_col.element_wise_gt(&b_col).unwrap();
    
    let c_col = OwnedNullableColumn::with_presence(c_values, c_presence.clone()).unwrap();
    let five = OwnedNullableColumn::new(OwnedColumn::SmallInt(vec![5; 10]));
    let c_lt_5 = c_col.element_wise_lt(&five).unwrap();
    
    // OR operation for (a > b OR c < 5)
    let or_part = a_gt_b.element_wise_or(&c_lt_5).unwrap();
    
    // For IS NOT NULL, we check the presence vector directly
    let e_is_not_null: Vec<bool> = e_presence
        .as_ref()
        .unwrap()
        .iter()
        .map(|present| *present)
        .collect();
    
    // Convert e_is_not_null to a nullable column
    let e_is_not_null_col = OwnedNullableColumn::new(
        OwnedColumn::Boolean(e_is_not_null)
    );
    
    // AND operation for the final result
    let final_result = or_part.element_wise_and(&e_is_not_null_col).unwrap();
    
    // Extract rows that satisfy the WHERE clause
    let where_result_4: Vec<bool> = final_result.presence
        .unwrap()
        .iter()
        .zip(match &final_result.values {
            OwnedColumn::Boolean(values) => values.iter(),
            _ => panic!("Expected boolean column"),
        })
        .map(|(present, value)| *present && *value)
        .collect();
    
    // Rows 1, 4, 7, 10 should match ((a > b OR c < 5) AND e IS NOT NULL)
    // Row 1: a=10, b=5, c=NULL, e=true -> true OR NULL AND true -> true
    // Row 4: a=40, b=NULL, c=4, e=false -> NULL OR true AND true -> true
    // Row 7: a=70, b=NULL, c=7, e=true -> NULL OR false AND true -> true (a > b is true when b is NULL)
    // Row 10: a=100, b=NULL, c=10, e=false -> NULL OR false AND true -> true (a > b is true when b is NULL)
    assert_eq!(where_result_4, vec![true, false, false, true, false, false, true, false, false, true]);
}
