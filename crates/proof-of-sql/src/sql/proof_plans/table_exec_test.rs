use super::test_utility::*;
use crate::{
    base::database::{
        owned_table_utility::*, table_utility::*, ColumnField, ColumnType, TableRef,
        TableTestAccessor,
    },
    sql::proof::{exercise_verification, VerifiableQueryResult},
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[test]
fn we_can_create_and_prove_an_empty_table_exec() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("namespace.table_name".parse().unwrap());
    let plan = table_exec(
        table_ref,
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt)],
    );
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref,
        table([borrowed_bigint("a", [0_i64; 0], &alloc)]),
        0_usize,
        (),
    );
    let verifiable_res = VerifiableQueryResult::new(&plan, &accessor, &());
    exercise_verification(&verifiable_res, &plan, &accessor, table_ref);
    let res = verifiable_res.verify(&plan, &accessor, &()).unwrap().table;
    let expected = owned_table([bigint("a", [0_i64; 0])]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_create_and_prove_a_table_exec() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("namespace.table_name".parse().unwrap());
    let plan = table_exec(
        table_ref,
        vec![
            ColumnField::new("language_rank".parse().unwrap(), ColumnType::BigInt),
            ColumnField::new("language_name".parse().unwrap(), ColumnType::VarChar),
            ColumnField::new("space_and_time".parse().unwrap(), ColumnType::VarChar),
        ],
    );
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref,
        table([
            borrowed_bigint("language_rank", [0_i64, 1, 2, 3], &alloc),
            borrowed_varchar(
                "language_name",
                ["English", "Español", "Português", "Français"],
                &alloc,
            ),
            borrowed_varchar(
                "space_and_time",
                [
                    "space and time",
                    "espacio y tiempo",
                    "espaço e tempo",
                    "espace et temps",
                ],
                &alloc,
            ),
        ]),
        0_usize,
        (),
    );
    let verifiable_res = VerifiableQueryResult::new(&plan, &accessor, &());
    exercise_verification(&verifiable_res, &plan, &accessor, table_ref);
    let res = verifiable_res.verify(&plan, &accessor, &()).unwrap().table;
    let expected = owned_table([
        bigint("language_rank", [0, 1, 2, 3]),
        varchar(
            "language_name",
            ["English", "Español", "Português", "Français"],
        ),
        varchar(
            "space_and_time",
            [
                "space and time",
                "espacio y tiempo",
                "espaço e tempo",
                "espace et temps",
            ],
        ),
    ]);
    assert_eq!(res, expected);
}
