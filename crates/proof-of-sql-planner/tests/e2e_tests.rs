//! In this file we run end-to-end tests for Proof of SQL.
use ark_std::test_rng;
use bumpalo::Bump;
use datafusion::config::ConfigOptions;
use indexmap::{indexmap, IndexMap};
use proof_of_sql::{
    base::{
        commitment::CommitmentEvaluationProof,
        database::{
            owned_table_utility::*, table_utility::*, LiteralValue, OwnedTable, Table, TableRef,
            TableTestAccessor, TestAccessor,
        },
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    proof_primitive::dory::{
        DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::proof::VerifiableQueryResult,
};
use proof_of_sql_planner::sql_to_proof_plans;
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// Get a new `TableTestAccessor` with the provided tables
fn new_test_accessor<'a, CP: CommitmentEvaluationProof>(
    tables: &IndexMap<TableRef, Table<'a, CP::Scalar>>,
    prover_setup: CP::ProverPublicSetup<'a>,
) -> TableTestAccessor<'a, CP> {
    let mut accessor = TableTestAccessor::<CP>::new_empty_with_setup(prover_setup);
    for (table_ref, table) in tables {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }
    accessor
}

/// Test setup
///
/// # Panics
/// This function will panic if anything goes wrong
fn posql_end_to_end_test<'a, CP: CommitmentEvaluationProof>(
    sql: &str,
    tables: &IndexMap<TableRef, Table<'a, CP::Scalar>>,
    expected_results: &[OwnedTable<CP::Scalar>],
    prover_setup: CP::ProverPublicSetup<'a>,
    verifier_setup: CP::VerifierPublicSetup<'_>,
    params: &[LiteralValue],
) {
    // Get accessor
    let accessor: TableTestAccessor<'a, CP> = new_test_accessor(tables, prover_setup);
    let config = ConfigOptions::default();
    let statements = Parser::parse_sql(&GenericDialect {}, sql).unwrap();
    let plans = sql_to_proof_plans(&statements, &accessor, &config).unwrap();
    // Prove and verify the plans
    for (plan, expected) in plans.iter().zip(expected_results.iter()) {
        let res = VerifiableQueryResult::<CP>::new(plan, &accessor, &prover_setup, params).unwrap();
        let res = res
            .verify(plan, &accessor, &verifier_setup, params)
            .unwrap()
            .table;
        assert_eq!(res, expected.clone());
    }
}

/// Empty SQL should return no plans
#[test]
fn test_empty_sql() {
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        "",
        &indexmap! {},
        &[],
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// Test tableless SQL queries
#[test]
fn test_tableless_queries() {
    let sql = "select 1 + 1;
    select 'tableless' as res;
    select 'Chloe' as name, 13 as age
    union all
    select 'Margaret' as name, 2 as age;
    select $1::varchar, $2::bigint;
    select $1::varchar as name, $2::bigint as age;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {};
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([bigint("Int64(1) + Int64(1)", [2_i64])]),
        owned_table([varchar("res", ["tableless"])]),
        owned_table([
            varchar("name", ["Chloe", "Margaret"]),
            bigint("age", [13_i64, 2]),
        ]),
        owned_table([varchar("$1", ["Katy"]), bigint("$2", [0_i64])]),
        owned_table([varchar("name", ["Katy"]), bigint("age", [0_i64])]),
    ];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[
            LiteralValue::VarChar("Katy".to_string()),
            LiteralValue::BigInt(0),
        ],
    );
}

/// Test a simple SQL query
#[test]
fn test_simple_filter_queries() {
    let alloc = Bump::new();
    let sql = "select id, name from cats where age > 2;
    select * from cats;
    select name == $1 as name_eq from cats;
    select 2 * age as double_age from cats;
    select id, name from cats where age <> 2";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
                borrowed_tinyint("age", [13_i8, 2, 0, 4, 4], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            int("id", [1, 4, 5]),
            varchar("name", ["Chloe", "Lucy", "Prudence"]),
        ]),
        owned_table([
            int("id", [1, 2, 3, 4, 5]),
            varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"]),
            tinyint("age", [13_i8, 2, 0, 4, 4]),
        ]),
        owned_table([boolean("name_eq", [false, false, true, false, false])]),
        owned_table([decimal75("double_age", 39, 0, [26_i8, 4, 0, 8, 8])]),
        owned_table([
            int("id", [1, 3, 4, 5]),
            varchar("name", ["Chloe", "Katy", "Lucy", "Prudence"]),
        ]),
    ];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[LiteralValue::VarChar("Katy".to_string())],
    );
}

/// Test complex filter queries with nested filters using subqueries
#[test]
fn test_complex_filter_queries() {
    let alloc = Bump::new();
    let sql = "
        SELECT id, value FROM (SELECT * FROM data WHERE value > 10) WHERE id < 4;
        SELECT id, name FROM (SELECT * FROM pets WHERE age > 2) WHERE id < 4 OR name = $1;
        SELECT a, double_b as b FROM (SELECT b * 2 as double_b, a + 1 as a FROM numbers WHERE a >= 0) WHERE double_b == 100;
    ";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "data") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_int("value", [5, 12, 18, 8, 25], &alloc),
            ]
        ),
        TableRef::from_names(None, "pets") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Rex", "Whiskers", "Fido", "Fluffy", "Buddy"], &alloc),
                borrowed_tinyint("age", [3_i8, 5, 1, 7, 4], &alloc),
            ]
        ),
        TableRef::from_names(None, "numbers") => table(
            vec![
                borrowed_bigint("a", [1_i64, 2, -1, 3, 0, 5], &alloc),
                borrowed_bigint("b", [10_i64, 50, 20, 150, 50, 50], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([int("id", [2, 3]), int("value", [12, 18])]),
        owned_table([
            int("id", [1, 2, 5]),
            varchar("name", ["Rex", "Whiskers", "Buddy"]),
        ]),
        owned_table([
            decimal75("a", 20, 0, [3_i64, 1, 6]),
            decimal75("b", 39, 0, [100_i64, 100, 100]),
        ]),
    ];

    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[LiteralValue::VarChar("Buddy".to_string())],
    );
}

/// Test projection operation - selecting only specific columns
#[test]
fn test_projection() {
    let alloc = Bump::new();
    let sql = r"SELECT name, age FROM pets;
    SELECT name, age, $1::boolean as is_cute FROM pets;";

    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "pets") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4], &alloc),
                borrowed_varchar("name", ["Rex", "Whiskers", "Fido", "Fluffy"], &alloc),
                borrowed_tinyint("age", [3_i8, 5, 2, 7], &alloc),
                borrowed_varchar("type", ["Dog", "Cat", "Dog", "Cat"], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            varchar("name", ["Rex", "Whiskers", "Fido", "Fluffy"]),
            tinyint("age", [3_i8, 5, 2, 7]),
        ]),
        owned_table([
            varchar("name", ["Rex", "Whiskers", "Fido", "Fluffy"]),
            tinyint("age", [3_i8, 5, 2, 7]),
            boolean("is_cute", [true; 4]),
        ]),
    ];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[LiteralValue::Boolean(true)],
    );
}

/// Test projection operation with scale casts
#[test]
fn test_projection_scaling() {
    let alloc = Bump::new();
    let sql = r"SELECT a + b as res FROM tab;";

    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "tab") => table(
            vec![
                borrowed_decimal75("a", 5, 1, [1, 2, 3, 4], &alloc),
                borrowed_decimal75("b", 3, 2, [5, 6, 7, 8], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> =
        vec![owned_table([decimal75("res", 7, 2, [15, 26, 37, 48])])];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// Test slicing/limit operation - retrieving only a subset of rows
#[test]
fn test_slicing_limit() {
    let alloc = Bump::new();
    let sql = "SELECT * FROM products LIMIT 2;";

    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "products") => table(
            vec![
                borrowed_int("id", [101, 102, 103, 104, 105], &alloc),
                borrowed_varchar("name", ["Laptop", "Phone", "Tablet", "Monitor", "Keyboard"], &alloc),
                borrowed_int("price", [1200, 800, 500, 300, 100], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([
        int("id", [101, 102]),
        varchar("name", ["Laptop", "Phone"]),
        int("price", [1200, 800]),
    ])];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// Test GROUP BY queries
#[test]
fn test_group_by() {
    let alloc = Bump::new();
    let sql = "select human_id, count(1) from cats group by human_id;
    select human_id, count(1) as num_cats from cats group by human_id;
    select human_id, sum(weight), count(1) from cats group by human_id;
    select human_id, sum(weight), count(1) as num_cats from cats group by human_id;
    select human_id, sum(weight) as total_weight, count(1) as num_cats from cats group by human_id;
    select human_id, sum(2 * weight), count(1) from cats group by human_id;
    select human_id, sum(2 * weight + 1) as total_transformed_weight, count(1) from cats group by human_id;
    select human_id, sum(2 * weight + $1::bigint) as total_transformed_weight, count(1) from cats group by human_id;
    select sum(2 * weight + 1) as total_transformed_weight, count(1) as num_cats from cats;
    select count(1) as num_cats from cats;
    select count(1) from cats;
    select count(id) from cats;
    select count(*) from cats;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
                borrowed_varchar("human", ["Cassia", "Cassia", "Cassia", "Gretta", "Gretta"], &alloc),
                borrowed_int("human_id", [1, 1, 1, 2, 2], &alloc),
                borrowed_decimal75("weight", 3, 1, [145, 75, 20, 45, 55], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            int("human_id", [1, 2]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([int("human_id", [1, 2]), bigint("num_cats", [3_i64, 2])]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("SUM(cats.weight)", 3, 1, [240, 100]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("SUM(cats.weight)", 3, 1, [240, 100]),
            bigint("num_cats", [3_i64, 2]),
        ]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("total_weight", 3, 1, [240, 100]),
            bigint("num_cats", [3_i64, 2]),
        ]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("SUM(Int64(2) * cats.weight)", 24, 1, [480, 200]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("total_transformed_weight", 25, 1, [510, 220]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            int("human_id", [1, 2]),
            decimal75("total_transformed_weight", 25, 1, [540, 240]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            decimal75("total_transformed_weight", 25, 1, [730]),
            bigint("num_cats", [5_i64]),
        ]),
        owned_table([bigint("num_cats", [5_i64])]),
        owned_table([bigint("COUNT(Int64(1))", [5_i64])]),
        owned_table([bigint("COUNT(cats.id)", [5_i64])]),
        owned_table([bigint("COUNT(*)", [5_i64])]),
    ];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[LiteralValue::BigInt(2)],
    );
}

#[test]
fn test_coin() {
    let alloc = Bump::new();
    let sql = "SELECT 
    SUM( 
      (
        CAST (to_address = $1 as bigint)
        - CAST (from_address = $1 as bigint)
      )
      * value
      * CAST(timestamp AS bigint)
    ) AS weighted_value,
    SUM( 
      (
        CAST (to_address = $1 as bigint)
        - CAST (from_address = $1 as bigint)
      )
      * value
    ) AS total_balance,
    COUNT(1) AS num_transactions
    FROM transactions;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "transactions") => table(
            vec![
                borrowed_varchar("from_address", ["0x1", "0x2", "0x3", "0x2", "0x1"], &alloc),
                borrowed_varchar("to_address", ["0x2", "0x3", "0x1", "0x3", "0x2"], &alloc),
                borrowed_decimal75("value", 75, 0, [100, 200, 300, 400, 500], &alloc),
                borrowed_timestamptz("timestamp", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), [1, 2, 3, 4, 4], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([
        decimal75("weighted_value", 75, 0, [100]),
        decimal75("total_balance", 75, 0, [0]),
        bigint("num_transactions", [5_i64]),
    ])];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[LiteralValue::VarChar("0x2".to_string())],
    );
}

#[test]
fn test_join() {
    let alloc = Bump::new();
    let sql = "SELECT column1, column2, column3 FROM table1 JOIN table2 ON table1.common_column = table2.common_column JOIN table3 on table3.another_column = table2.another_column;
    SELECT table1.common_column*table2.another_column as product FROM table1 JOIN table2 ON table1.common_column = table2.common_column;
    SELECT table2.common_column+table3.another_column as sum_result FROM table3 JOIN table2 ON table2.another_column = table3.another_column;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "table1") => table(
            vec![
                borrowed_int("common_column", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("column1", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
            ]
        ),
        TableRef::from_names(None, "table2") => table(
            vec![
                borrowed_int("common_column", [2, 3, 5, 4], &alloc),
                borrowed_int("another_column", [7, 8, 10, 10], &alloc),
                borrowed_varchar("column2", ["Test", "Some", "Creamy", "Chocolate"], &alloc),
            ]
        ),
        TableRef::from_names(None, "table3") => table(
            vec![
                borrowed_int("another_column", [8, 10, 6], &alloc),
                borrowed_varchar("column3", ["Puppy", "Dog", "Eyes"], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            varchar("column1", ["Katy", "Lucy", "Prudence"]),
            varchar("column2", ["Some", "Chocolate", "Creamy"]),
            varchar("column3", ["Puppy", "Dog", "Dog"]),
        ]),
        owned_table([decimal75("product", 21, 0, [14, 24, 40, 50])]),
        owned_table([decimal75("sum_result", 11, 0, [11, 15, 14])]),
    ];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

#[test]
fn test_corporate_query() {
    let alloc = Bump::new();
    let sql = "SELECT p.project_id,
       p.budget,
       p.percent_complete,
       spent.budget_spent,
       (p.budget - spent.budget_spent) AS budget_remaining,
       (p.budget * p.percent_complete) AS expected_spent
FROM projects p
JOIN (
    SELECT eph.project_id,
           SUM(e.hourly_wage * eph.hours_logged) AS budget_spent
    FROM employee_project_hours eph
    JOIN employees e
      ON eph.employee_id = e.employee_id
    GROUP BY eph.project_id
) spent
  ON p.project_id = spent.project_id;
";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "employees") => table(
            vec![
                borrowed_smallint("employee_id", [101i16,102,103,104,105,106,107,108,109,110], &alloc),
                borrowed_smallint("department_id", [1i16, 1, 1, 2, 2, 2, 3, 3, 3, 3], &alloc),
                borrowed_smallint("hourly_wage", [40i16, 45, 38, 30, 32, 35, 50, 55, 60, 100], &alloc),
            ]
        ),
        TableRef::from_names(None, "departments") => table(
            vec![
                borrowed_int("department_id", [1, 2, 3], &alloc),
                borrowed_varchar("department_name", ["Engineering", "HR", "Marketing"], &alloc),
            ]
        ),
        TableRef::from_names(None, "projects") => table(
            vec![
                borrowed_smallint("project_id", [1i16,2,3,4,5,6,7,8,9,10], &alloc),
                borrowed_bigint("budget", [30_000, 50_000, 20_000, 60_000, 100_000, 40_000, 20_000, 30_000, 20_000, 90_000], &alloc),
                borrowed_decimal75("percent_complete", 2, 2, [25, 60, 40, 75, 10, 95, 50, 80, 20, 33], &alloc),
            ]
        ),
        TableRef::from_names(None, "employee_project_hours") => table(
            vec![
                borrowed_smallint("employee_id", [101i16,102,102,103,104,105,105,106,107,108,108,109,110], &alloc),
                borrowed_smallint("project_id", [1i16, 1, 2, 2, 3, 3, 7, 8, 4, 10, 9, 6, 5], &alloc),
                borrowed_smallint("hours_logged", [160i16, 100, 60, 160, 160, 90, 70, 160, 160, 120, 40, 160, 160], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([
        smallint("project_id", [1i16, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        bigint(
            "budget",
            [
                30_000, 50_000, 20_000, 60_000, 100_000, 40_000, 20_000, 30_000, 20_000, 90_000,
            ],
        ),
        decimal75(
            "percent_complete",
            2,
            2,
            [25, 60, 40, 75, 10, 95, 50, 80, 20, 33],
        ),
        bigint(
            "budget_spent",
            [
                10_900, 8_780, 7_680, 8_000, 16_000, 9_600, 2_240, 5_600, 2_200, 6_600,
            ],
        ),
        decimal75(
            "budget_remaining",
            20,
            0,
            [
                19_100, 41_220, 12_320, 52_000, 84_000, 30_400, 17_760, 24_400, 17_800, 83_400,
            ],
        ),
        decimal75(
            "expected_spent",
            23,
            2,
            [
                750_000, 3_000_000, 800_000, 4_500_000, 1_000_000, 3_800_000, 1_000_000, 2_400_000,
                400_000, 2_970_000,
            ],
        ),
    ])];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

#[test]
fn test_union() {
    let alloc = Bump::new();
    // WE do not yet support regular UNION
    let sql = "SELECT column1 FROM table1 UNION ALL SELECT column2 FROM table2;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "table1") => table(
            vec![
                borrowed_varchar("column1", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
            ]
        ),
        TableRef::from_names(None, "table2") => table(
            vec![
                borrowed_varchar("column2", ["Test", "Some", "Creamy", "Chocolate"], &alloc),
            ]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([varchar(
        "column1",
        [
            "Chloe",
            "Margaret",
            "Katy",
            "Lucy",
            "Prudence",
            "Test",
            "Some",
            "Creamy",
            "Chocolate",
        ],
    )])];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

#[test]
fn test_implicit_casts() {
    let alloc = Bump::new();
    let sql = "SELECT a<b as compared, a*b as product from sxt.table where a+b>3.5;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(Some("sxt"), "table") => table(
            vec![
                borrowed_int("a", [1, 2, 3, 4, 5], &alloc),
                borrowed_decimal75("b", 3, 1, [300, 400, 10, 2, -30], &alloc),
            ]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([
        boolean("compared", [true, true, false, false]),
        decimal75("product", 14, 1, [300, 800, 30, 8]),
    ])];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

#[test]
fn test_in_list() {
    let alloc = Bump::new();
    // Numeric `IN`/`NOT IN`/single-value, plus varchar `IN`/`NOT IN` (which lowers to an
    // OR-chain of equalities rather than the product form).
    let sql = "SELECT id, name FROM sxt.cats WHERE id IN (1, 3, 5);
        SELECT id FROM sxt.cats WHERE id NOT IN (1, 3, 5);
        SELECT name FROM sxt.cats WHERE id IN (4);
        SELECT id FROM sxt.cats WHERE name IN ('Chloe', 'Katy');
        SELECT id FROM sxt.cats WHERE name NOT IN ('Chloe', 'Katy', 'Lucy');";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(Some("sxt"), "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
            ]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            int("id", [1, 3, 5]),
            varchar("name", ["Chloe", "Katy", "Prudence"]),
        ]),
        owned_table([int("id", [2, 4])]),
        owned_table([varchar("name", ["Lucy"])]),
        owned_table([int("id", [1, 3])]),
        owned_table([int("id", [2, 5])]),
    ];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// `IN` with no matches (empty result), `IN` composed with another predicate (two `IN`s
/// `AND`ed), and a longer list.
#[test]
fn test_in_list_more_cases() {
    let alloc = Bump::new();
    let sql = "SELECT id FROM sxt.cats WHERE id IN (100, 200);
        SELECT id FROM sxt.cats WHERE id IN (1, 3, 5) AND name IN ('Chloe', 'Katy');
        SELECT id FROM sxt.cats WHERE id IN (1, 2, 3, 4, 5, 6);";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(Some("sxt"), "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
            ]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([int("id", [0; 0])]),          // no match -> empty
        owned_table([int("id", [1, 3])]),          // id IN (1,3,5) AND name IN ('Chloe','Katy')
        owned_table([int("id", [1, 2, 3, 4, 5])]), // 6-element list -> all rows match
    ];
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// `IN` over a decimal column, exercising the scale-cast / product path with a non-integer type.
#[test]
fn test_in_list_decimal() {
    let alloc = Bump::new();
    let sql = "SELECT id FROM sxt.items WHERE amount IN (1.5, 3.0);";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(Some("sxt"), "items") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4], &alloc),
                // amount = 1.5, 2.5, 1.5, 3.0  (decimal with scale 1)
                borrowed_decimal75("amount", 3, 1, [15, 25, 15, 30], &alloc),
            ]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([int("id", [1, 3, 4])])];
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// A long numeric `IN` list proves and verifies fine. In the product form the result type's
/// precision saturates at 75 (`(p1 + p2 + 1).min(75)`, so it never errors), and integer factors
/// keep scale 0, so there is no practical length limit for numeric lists. The membership zero-test
/// stays correct regardless of the precision metadata. (Only very long *decimal* lists could hit
/// `i8` scale accumulation.)
#[test]
fn test_in_list_long_numeric_list() {
    let alloc = Bump::new();
    let sql = "SELECT id FROM sxt.cats WHERE id IN (2, 4, 6, 8, 10, 12, 14, 16, 18, 20);";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(Some("sxt"), "cats") => table(
            vec![borrowed_int("id", [1, 2, 3, 4, 5], &alloc)]
        ),
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([int("id", [2, 4])])];
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// Test BETWEEN and NOT BETWEEN operators, including inclusive boundary behaviour
#[test]
fn test_between_operator() {
    let alloc = Bump::new();
    let sql = "SELECT id, score FROM students WHERE score BETWEEN 60 AND 90;
    SELECT id, score FROM students WHERE score NOT BETWEEN 60 AND 90;
    SELECT id, score FROM students WHERE score BETWEEN 90 AND 90;";

    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "students") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_bigint("score", [45_i64, 60, 75, 90, 95], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        // score >= 60 AND score <= 90  →  rows 2, 3, 4
        owned_table([int("id", [2, 3, 4]), bigint("score", [60_i64, 75, 90])]),
        // score < 60 OR score > 90  →  rows 1, 5
        owned_table([int("id", [1, 5]), bigint("score", [45_i64, 95])]),
        // exact single-value range  →  row 4 only
        owned_table([int("id", [4]), bigint("score", [90_i64])]),
    ];

    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}

/// Test BETWEEN combined with AND / OR filters
#[test]
fn test_between_combined_with_other_filters() {
    let alloc = Bump::new();
    let sql =
        "SELECT id, name FROM employees WHERE salary BETWEEN 50000 AND 80000 AND department = 'Engineering';
    SELECT id, name FROM employees WHERE salary BETWEEN 50000 AND 80000 OR age < 25;";

    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "employees") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Alice", "Bob", "Carol", "Dave", "Eve"], &alloc),
                borrowed_bigint("salary", [45_000_i64, 60_000, 75_000, 90_000, 55_000], &alloc),
                borrowed_varchar("department", ["HR", "Engineering", "Engineering", "Engineering", "HR"], &alloc),
                borrowed_tinyint("age", [30_i8, 24, 35, 28, 22], &alloc),
            ]
        )
    };

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        // salary in [50000,80000]: ids 2,3,5  AND  department='Engineering': ids 2,3
        owned_table([int("id", [2, 3]), varchar("name", ["Bob", "Carol"])]),
        // salary in [50000,80000]: ids 2,3,5  OR  age < 25 (ids 2,5)  →  union: ids 2,3,5
        owned_table([
            int("id", [2, 3, 5]),
            varchar("name", ["Bob", "Carol", "Eve"]),
        ]),
    ];

    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}
