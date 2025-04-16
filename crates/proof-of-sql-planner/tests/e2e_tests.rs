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
use proof_of_sql_planner::{
    postprocessing::PostprocessingStep, sql_to_proof_plans, sql_to_proof_plans_with_postprocessing,
};
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

/// # Panics
/// This function will panic if anything goes wrong
fn posql_end_to_end_test_with_postprocessing<'a, CP: CommitmentEvaluationProof>(
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
    let plan_with_postprocessings =
        sql_to_proof_plans_with_postprocessing(&statements, &accessor, &config).unwrap();
    for (plan_with_postprocessing, expected) in plan_with_postprocessings
        .iter()
        .zip(expected_results.iter())
    {
        // Prove and verify the plans
        let plan = plan_with_postprocessing.plan();
        let res = VerifiableQueryResult::<CP>::new(plan, &accessor, &prover_setup, params).unwrap();
        let raw_table = res
            .verify(plan, &accessor, &verifier_setup, params)
            .unwrap()
            .table;
        // Apply postprocessing
        let transformed_table = plan_with_postprocessing
            .postprocessing()
            .map_or(raw_table.clone(), |postproc| {
                postproc.apply(raw_table).unwrap()
            });
        assert_eq!(transformed_table, expected.clone());
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
    select 2 * age as double_age from cats";
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
    let sql = "select human, count(1) from cats group by human;
    select human, count(1) as num_cats from cats group by human;
    select human, sum(weight), count(1) from cats group by human;
    select human, sum(weight), count(1) as num_cats from cats group by human;
    select human, sum(weight) as total_weight, count(1) as num_cats from cats group by human;
    select human, sum(2 * weight), count(1) from cats group by human;
    select human, sum(2 * weight + 1) as total_transformed_weight, count(1) from cats group by human;
    select human, sum(2 * weight + $1::bigint) as total_transformed_weight, count(1) from cats group by human;
    select sum(2 * weight + 1) as total_transformed_weight, count(1) as num_cats from cats;
    select count(1) as num_cats from cats;
    select count(1) from cats;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
                borrowed_varchar("human", ["Cassia", "Cassia", "Cassia", "Gretta", "Gretta"], &alloc),
                borrowed_decimal75("weight", 3, 1, [145, 75, 20, 45, 55], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            bigint("num_cats", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("SUM(cats.weight)", 3, 1, [240, 100]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("SUM(cats.weight)", 3, 1, [240, 100]),
            bigint("num_cats", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("total_weight", 3, 1, [240, 100]),
            bigint("num_cats", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("SUM(Int64(2) * cats.weight)", 24, 1, [480, 200]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("total_transformed_weight", 25, 1, [510, 220]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            varchar("human", ["Cassia", "Gretta"]),
            decimal75("total_transformed_weight", 25, 1, [540, 240]),
            bigint("COUNT(Int64(1))", [3_i64, 2]),
        ]),
        owned_table([
            decimal75("total_transformed_weight", 25, 1, [730]),
            bigint("num_cats", [5_i64]),
        ]),
        owned_table([bigint("num_cats", [5_i64])]),
        owned_table([bigint("COUNT(Int64(1))", [5_i64])]),
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

// Test GROUP BY queries with postprocessing
#[test]
fn test_group_by_with_postprocessing() {
    let alloc = Bump::new();
    let sql = "select human, 2*count(1) as double_cat_count from cats group by human;
    select human, 2*count(1) from cats group by human;";
    let tables: IndexMap<TableRef, Table<DoryScalar>> = indexmap! {
        TableRef::from_names(None, "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], &alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], &alloc),
                borrowed_varchar("human", ["Cassia", "Cassia", "Cassia", "Gretta", "Gretta"], &alloc),
                borrowed_decimal75("weight", 3, 1, [145, 75, 20, 45, 55], &alloc),
            ]
        )
    };
    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![
        owned_table([
            varchar("cats.human", ["Cassia", "Gretta"]),
            bigint("double_cat_count", [6_i64, 4]),
        ]),
        owned_table([
            varchar("cats.human", ["Cassia", "Gretta"]),
            bigint("Int64(2) * COUNT(Int64(1))", [6_i64, 4]),
        ]),
    ];
    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test_with_postprocessing::<DynamicDoryEvaluationProof>(
        sql,
        &tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
        &[],
    );
}
