//! In this file we run end-to-end tests for Proof of SQL.
#![cfg_attr(test, allow(clippy::missing_panics_doc))]
use ark_std::test_rng;
use bumpalo::Bump;
use datafusion::{catalog::TableReference, common::DFSchema, config::ConfigOptions};
use indexmap::{indexmap, IndexMap};
use proof_of_sql::{
    base::{
        commitment::CommitmentEvaluationProof,
        database::{
            owned_table_utility::*, table_utility::*, OwnedTable, Table, TableRef,
            TableTestAccessor, TestAccessor,
        },
    },
    proof_primitive::dory::{
        DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::proof::VerifiableQueryResult,
};
use proof_of_sql_planner::{
    column_fields_to_schema, postprocessing::PostprocessingStep, sql_to_proof_plans,
    sql_to_proof_plans_with_postprocessing, PlannerResult, PoSqlContextProvider,
};

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

/// Get the schemas of the provided tables
fn get_schemas<CP: CommitmentEvaluationProof>(
    tables: &IndexMap<TableRef, Table<'_, CP::Scalar>>,
) -> PlannerResult<IndexMap<TableReference, DFSchema>> {
    tables
        .iter()
        .map(
            |(table_ref, table)| -> PlannerResult<(TableReference, DFSchema)> {
                let table_reference = TableReference::from(table_ref.to_string().as_str());
                let schema = column_fields_to_schema(table.schema());
                let df_schema =
                    DFSchema::try_from_qualified_schema(table_reference.clone(), &schema)?;
                Ok((table_reference, df_schema))
            },
        )
        .collect::<PlannerResult<IndexMap<_, _>>>()
}

/// Test setup
///
/// # Panics
/// This function will panic if anything goes wrong
fn posql_end_to_end_test<'a, CP: CommitmentEvaluationProof>(
    sql: &str,
    tables: IndexMap<TableRef, Table<'a, CP::Scalar>>,
    expected_results: &[OwnedTable<CP::Scalar>],
    prover_setup: CP::ProverPublicSetup<'a>,
    verifier_setup: CP::VerifierPublicSetup<'_>,
) {
    // Get accessor
    let accessor: TableTestAccessor<'a, CP> = new_test_accessor(&tables, prover_setup);
    let schemas = get_schemas::<CP>(&tables).unwrap();
    let context_provider = PoSqlContextProvider::new(tables);
    let config = ConfigOptions::default();
    let plans = sql_to_proof_plans(sql, &context_provider, &schemas, &config).unwrap();
    // Prove and verify the plans
    for (plan, expected) in plans.iter().zip(expected_results.iter()) {
        let res = VerifiableQueryResult::<CP>::new(plan, &accessor, &prover_setup);
        let res = res.verify(plan, &accessor, &verifier_setup).unwrap().table;
        assert_eq!(res, expected.clone());
    }
}

/// # Panics
/// This function will panic if anything goes wrong
fn posql_end_to_end_test_with_postprocessing<'a, CP: CommitmentEvaluationProof>(
    sql: &str,
    tables: IndexMap<TableRef, Table<'a, CP::Scalar>>,
    expected_results: &[OwnedTable<CP::Scalar>],
    prover_setup: CP::ProverPublicSetup<'a>,
    verifier_setup: CP::VerifierPublicSetup<'_>,
) {
    // Get accessor
    let accessor: TableTestAccessor<'a, CP> = new_test_accessor(&tables, prover_setup);
    let schemas = get_schemas::<CP>(&tables).unwrap();
    let context_provider = PoSqlContextProvider::new(tables);
    let config = ConfigOptions::default();
    let plan_with_postprocessings =
        sql_to_proof_plans_with_postprocessing(sql, &context_provider, &schemas, &config).unwrap();
    for (plan_with_postprocessing, expected) in plan_with_postprocessings
        .iter()
        .zip(expected_results.iter())
    {
        // Prove and verify the plans
        let plan = plan_with_postprocessing.plan();
        let res = VerifiableQueryResult::<CP>::new(plan, &accessor, &prover_setup);
        let raw_table = res.verify(plan, &accessor, &verifier_setup).unwrap().table;
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
        indexmap! {},
        &[],
        &prover_setup,
        &verifier_setup,
    );
}

/// Test a simple SQL query
#[test]
fn test_simple_filter_queries() {
    let alloc = Bump::new();
    let sql = "select id, name from cats where age > 2;
    select * from cats;";
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
    ];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
    );
}

/// Test projection operation - selecting only specific columns
#[test]
fn test_projection() {
    let alloc = Bump::new();
    let sql = "SELECT name, age FROM pets;";

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

    let expected_results: Vec<OwnedTable<DoryScalar>> = vec![owned_table([
        varchar("name", ["Rex", "Whiskers", "Fido", "Fluffy"]),
        tinyint("age", [3_i8, 5, 2, 7]),
    ])];

    // Create public parameters for DynamicDoryEvaluationProof
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    posql_end_to_end_test::<DynamicDoryEvaluationProof>(
        sql,
        tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
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
        tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
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
        tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
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
        tables,
        &expected_results,
        &prover_setup,
        &verifier_setup,
    );
}
