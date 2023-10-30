use crate::{
    base::database::{OwnedColumn, OwnedTable, OwnedTableError},
    owned_table,
};
use indexmap::IndexMap;
use proofs_sql::Identifier;

#[test]
fn we_can_create_an_owned_table_with_no_columns() {
    let table = OwnedTable::try_new(IndexMap::new()).unwrap();
    assert_eq!(table.num_columns(), 0);
}
#[test]
fn we_can_create_an_empty_owned_table() {
    let owned_table = owned_table!(
        "a" => [0_i64; 0],
        "b" => [0_i128; 0],
        "c" => ["0"; 0],
    );
    let mut table = IndexMap::new();
    table.insert(
        Identifier::try_new("a").unwrap(),
        OwnedColumn::BigInt(vec![]),
    );
    table.insert(
        Identifier::try_new("b").unwrap(),
        OwnedColumn::Int128(vec![]),
    );
    table.insert(
        Identifier::try_new("c").unwrap(),
        OwnedColumn::VarChar(vec![]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_can_create_an_owned_table_with_data() {
    let owned_table = owned_table!(
        "a" => [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
        "b" => [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
        "c" => ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
    );
    let mut table = IndexMap::new();
    table.insert(
        Identifier::try_new("a").unwrap(),
        OwnedColumn::BigInt(vec![0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
    );
    table.insert(
        Identifier::try_new("b").unwrap(),
        OwnedColumn::Int128(vec![0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
    );
    table.insert(
        Identifier::try_new("c").unwrap(),
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
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let owned_table_a = owned_table!(
        "a" => [0_i64; 0],
        "b" => [0_i128; 0],
        "c" => ["0"; 0],
    );
    let owned_table_b = owned_table!(
        "b" => [0_i128; 0],
        "a" => [0_i64; 0],
        "c" => ["0"; 0],
    );
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let owned_table_a = owned_table!(
        "a" => [0_i64],
        "b" => [0_i128],
        "c" => ["0"],
    );
    let owned_table_b = owned_table!(
        "a" => [1_i64],
        "b" => [0_i128],
        "c" => ["0"],
    );
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_cannot_create_an_owned_table_with_differing_column_lengths() {
    assert!(matches!(
        OwnedTable::try_from_iter([
            ("a".parse().unwrap(), OwnedColumn::BigInt(vec![0])),
            ("b".parse().unwrap(), OwnedColumn::BigInt(vec![])),
        ]),
        Err(OwnedTableError::ColumnLengthMismatch)
    ));
}
