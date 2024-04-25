use crate::{
    base::{
        database::{OwnedColumn, OwnedTable, OwnedTableError},
        scalar::Curve25519Scalar,
    },
    owned_table,
    proof_primitive::dory::DoryScalar,
};
use indexmap::IndexMap;
use proofs_sql::Identifier;

#[test]
fn we_can_create_an_owned_table_with_no_columns() {
    let table = OwnedTable::<Curve25519Scalar>::try_new(IndexMap::new()).unwrap();
    assert_eq!(table.num_columns(), 0);
}
#[test]
fn we_can_create_an_empty_owned_table() {
    let owned_table = owned_table!(
        "bigint" => [0_i64; 0],
        "decimal" => [0_i128; 0],
        "varchar" => ["0"; 0],
        "scalar" => [DoryScalar::from(0); 0],
        "boolean" => [true; 0],
    );
    let mut table = IndexMap::new();
    table.insert(
        Identifier::try_new("bigint").unwrap(),
        OwnedColumn::BigInt(vec![]),
    );
    table.insert(
        Identifier::try_new("decimal").unwrap(),
        OwnedColumn::Int128(vec![]),
    );
    table.insert(
        Identifier::try_new("varchar").unwrap(),
        OwnedColumn::VarChar(vec![]),
    );
    table.insert(
        Identifier::try_new("scalar").unwrap(),
        OwnedColumn::Scalar(vec![]),
    );
    table.insert(
        Identifier::try_new("boolean").unwrap(),
        OwnedColumn::Boolean(vec![]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_can_create_an_owned_table_with_data() {
    let owned_table = owned_table!(
        "bigint" => [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
        "decimal" => [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
        "varchar" => ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
        "scalar" => [DoryScalar::from(0), 1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), 8.into()],
        "boolean" => [true, false, true, false, true, false, true, false, true],
    );
    let mut table = IndexMap::new();
    table.insert(
        Identifier::try_new("bigint").unwrap(),
        OwnedColumn::BigInt(vec![0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
    );
    table.insert(
        Identifier::try_new("decimal").unwrap(),
        OwnedColumn::Int128(vec![0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
    );
    table.insert(
        Identifier::try_new("varchar").unwrap(),
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
        Identifier::try_new("scalar").unwrap(),
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
        Identifier::try_new("boolean").unwrap(),
        OwnedColumn::Boolean(vec![
            true, false, true, false, true, false, true, false, true,
        ]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let owned_table_a: OwnedTable<Curve25519Scalar> = owned_table!(
        "a" => [0_i64; 0],
        "b" => [0_i128; 0],
        "c" => ["0"; 0],
        "d" => [false; 0],
    );
    let owned_table_b: OwnedTable<Curve25519Scalar> = owned_table!(
        "d" => [false; 0],
        "b" => [0_i128; 0],
        "a" => [0_i64; 0],
        "c" => ["0"; 0],
    );
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let owned_table_a: OwnedTable<DoryScalar> = owned_table!(
        "a" => [0_i64],
        "b" => [0_i128],
        "c" => ["0"],
        "d" => [true]
    );
    let owned_table_b: OwnedTable<DoryScalar> = owned_table!(
        "a" => [1_i64],
        "b" => [0_i128],
        "c" => ["0"],
        "d" => [true]
    );
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_cannot_create_an_owned_table_with_differing_column_lengths() {
    assert!(matches!(
        OwnedTable::<Curve25519Scalar>::try_from_iter([
            ("a".parse().unwrap(), OwnedColumn::BigInt(vec![0])),
            ("b".parse().unwrap(), OwnedColumn::BigInt(vec![])),
        ]),
        Err(OwnedTableError::ColumnLengthMismatch)
    ));
}
