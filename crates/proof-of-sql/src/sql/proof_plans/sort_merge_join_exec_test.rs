use super::test_utility::*;
use crate::{
    base::database::{
        owned_table_utility::*, table_utility::*, ColumnType, TableTestAccessor, TestAccessor,
    },
    sql::proof::{exercise_verification, VerifiableQueryResult},
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7], &alloc),
        borrowed_varchar(
            "human",
            ["Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik"],
            &alloc,
        ),
    ]);
    let table_right = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left, left, 0);
    accessor.add_table(table_right, right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 4]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Lucy"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Gretta"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left, left, 0);
    accessor.add_table(table_right, right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[allow(clippy::too_many_lines)]
#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_sort_merge_join_if_one_or_both_tables_have_no_rows(
) {
    // Left table has no rows but right table has rows
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("name", [""; 0], &alloc),
    ]);
    let table_left = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left, left, 0);
    accessor.add_table(table_right, right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, table_right);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);

    // Right table has no rows but left table has rows
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left, left, 0);
    accessor.add_table(table_right, right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);

    // Both tables have no rows
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("name", [""; 0], &alloc),
    ]);
    let table_left = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left, left, 0);
    accessor.add_table(table_right, right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);
}
