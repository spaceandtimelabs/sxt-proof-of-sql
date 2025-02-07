use super::test_utility::*;
use crate::{
    base::database::{
        owned_table_utility::*, table_utility::*, ColumnType, TableRef, TableTestAccessor,
        TestAccessor,
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::test_utility::*,
    },
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
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7], &alloc),
        borrowed_varchar(
            "human",
            ["Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik"],
            &alloc,
        ),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 4]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Lucy"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Gretta"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_complex_query_involving_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let cats = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 29, 20, 21], &alloc),
        borrowed_varchar(
            "name",
            [
                "Chloe", "Margaret", "Prudence", "Lucy", "Pepper", "Rocky", "Whiskers", "Mittens",
                "Felix",
            ],
            &alloc,
        ),
    ]);
    let table_cats: TableRef = "sxt.cats".parse().unwrap();
    let cat_details = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7, 5, 6], &alloc),
        borrowed_varchar(
            "human",
            [
                "Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik", "Gretta", "Gretta",
            ],
            &alloc,
        ),
    ]);
    let table_cat_details: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_cats.clone(), cats, 0);
    accessor.add_table(table_cat_details.clone(), cat_details, 0);
    let ast = slice_exec(
        sort_merge_join(
            filter(
                cols_expr_plan(&table_cats, &["id", "name"], &accessor),
                tab(&table_cats),
                lte(column(&table_cats, "id", &accessor), const_int128(20)),
            ),
            filter(
                cols_expr_plan(&table_cat_details, &["id", "human"], &accessor),
                tab(&table_cat_details),
                not(equal(
                    column(&table_cat_details, "human", &accessor),
                    const_varchar("Gretta"),
                )),
            ),
            vec![0],
            vec![0],
            vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
        ),
        2,
        Some(3),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &table_cats);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [2_i64, 2]),
        varchar("name", ["Margaret", "Margaret"]),
        varchar("human", ["Cassia", "Ian"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
#[allow(clippy::too_many_lines)]
fn we_can_prove_and_get_the_correct_result_from_a_complex_query_involving_two_sort_merge_joins() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let cats = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 10, 29, 20, 21], &alloc),
        borrowed_varchar(
            "name",
            [
                "Chloe", "Margaret", "Prudence", "Lucy", "Pepper", "Rocky", "Nova", "Whiskers",
                "Mittens", "Felix",
            ],
            &alloc,
        ),
    ]);
    let table_cats: TableRef = "sxt.cats".parse().unwrap();
    let cat_human = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 10, 1, 2, 7, 5, 6], &alloc),
        borrowed_varchar(
            "human",
            [
                "Cassia", "Cassia", "Gretta", "Gretta", "Trevor", "Ian", "Ian", "Erik", "Gretta",
                "Gretta",
            ],
            &alloc,
        ),
        borrowed_varchar(
            "state",
            ["TX", "TX", "NC", "NC", "CO", "NC", "NC", "ND", "NC", "NC"],
            &alloc,
        ),
    ]);
    let table_cat_human: TableRef = "sxt.cat_human".parse().unwrap();
    let cat_vet = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 9, 8, 10], &alloc),
        borrowed_varchar(
            "hospital",
            [
                "Mint Hill",
                "Mint Hill",
                "Brown Creek",
                "Brown Creek",
                "Brown Creek",
                "Brown Creek",
                "Clear Creek",
                "Clear Creek",
                "Rock Creek",
            ],
            &alloc,
        ),
    ]);
    let table_cat_vet: TableRef = "sxt.cat_vet".parse().unwrap();
    accessor.add_table(table_cats.clone(), cats, 0);
    accessor.add_table(table_cat_human.clone(), cat_human, 0);
    accessor.add_table(table_cat_vet.clone(), cat_vet, 0);
    let ast = sort_merge_join(
        sort_merge_join(
            filter(
                cols_expr_plan(&table_cats, &["id", "name"], &accessor),
                tab(&table_cats),
                lte(column(&table_cats, "id", &accessor), const_int128(20)),
            ),
            filter(
                cols_expr_plan(&table_cat_human, &["id", "human", "state"], &accessor),
                tab(&table_cat_human),
                not(equal(
                    column(&table_cat_human, "human", &accessor),
                    const_varchar("Gretta"),
                )),
            ),
            vec![0],
            vec![0],
            vec![
                Ident::new("id"),
                Ident::new("name"),
                Ident::new("human"),
                Ident::new("state"),
            ],
        ),
        filter(
            cols_expr_plan(&table_cat_vet, &["id", "hospital"], &accessor),
            tab(&table_cat_vet),
            not(equal(
                column(&table_cat_vet, "hospital", &accessor),
                const_varchar("Clear Creek"),
            )),
        ),
        vec![0],
        vec![0],
        vec![
            Ident::new("id"),
            Ident::new("name"),
            Ident::new("human"),
            Ident::new("state"),
            Ident::new("hospital"),
        ],
    );

    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    exercise_verification(&verifiable_res, &ast, &accessor, &table_cats);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 10]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Nova"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Trevor"]),
        varchar("state", ["TX", "NC", "TX", "NC", "CO"]),
        varchar(
            "hospital",
            [
                "Mint Hill",
                "Mint Hill",
                "Mint Hill",
                "Mint Hill",
                "Rock Creek",
            ],
        ),
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
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
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
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &table_right);
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
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
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
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
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
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);
}
