//! Support-matrix probe for `CASE WHEN` queries.
use ark_std::test_rng;
use bumpalo::Bump;
use datafusion::config::ConfigOptions;
use indexmap::{indexmap, IndexMap};
use proof_of_sql::{
    base::database::{table_utility::*, Table, TableRef, TableTestAccessor, TestAccessor},
    proof_primitive::dory::{
        DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters,
    },
};
use proof_of_sql_planner::sql_to_proof_plans;
use sqlparser::{dialect::GenericDialect, parser::Parser};

fn cats_table<'a>(alloc: &'a Bump) -> IndexMap<TableRef, Table<'a, DoryScalar>> {
    indexmap! {
        TableRef::from_names(None, "cats") => table(
            vec![
                borrowed_int("id", [1, 2, 3, 4, 5], alloc),
                borrowed_varchar("name", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"], alloc),
                borrowed_int("human_id", [1, 1, 1, 2, 2], alloc),
                borrowed_decimal75("weight", 3, 1, [145, 75, 20, 45, 55], alloc),
                borrowed_boolean("adopted", [true, true, false, false, true], alloc),
            ]
        )
    }
}

/// A boolean-valued CASE inside WHERE works today: DataFusion rewrites it into
/// boolean algebra, e.g. `CASE WHEN c THEN a ELSE true END` -> `(c AND a) OR NOT c`,
/// before the planner lowers it. Prove + verify to pin this.
#[test]
fn test_boolean_case_in_where_clause_proves() {
    use proof_of_sql::{
        base::database::{owned_table_utility::*, OwnedTable},
        proof_primitive::dory::VerifierSetup,
        sql::proof::VerifiableQueryResult,
    };
    let alloc = Bump::new();
    let tables = cats_table(&alloc);
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let mut accessor =
        TableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    for (table_ref, table) in &tables {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }
    let config = ConfigOptions::default();
    let statements = Parser::parse_sql(
        &GenericDialect {},
        // human 1's cats only if adopted; human 2's cats unconditionally
        "SELECT id FROM cats WHERE CASE WHEN human_id = 1 THEN adopted ELSE true END",
    )
    .unwrap();
    let plans = sql_to_proof_plans(&statements, &accessor, &config).unwrap();
    let res = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        &plans[0],
        &accessor,
        &&prover_setup,
        &[],
    )
    .unwrap();
    let res = res
        .verify(&plans[0], &accessor, &&verifier_setup, &[])
        .unwrap()
        .table;
    // adopted: [t, t, f, f, t]; human_id: [1, 1, 1, 2, 2] -> ids 1, 2 (adopted, human 1), 4, 5 (human 2)
    let expected: OwnedTable<DoryScalar> = owned_table([int("id", [1, 2, 4, 5])]);
    assert_eq!(res, expected);
}

/// Convert, prove and verify each query against the cats table, asserting results.
fn assert_case_queries_prove_and_verify(
    sql: &str,
    expected_results: &[proof_of_sql::base::database::OwnedTable<DoryScalar>],
) {
    use proof_of_sql::{proof_primitive::dory::VerifierSetup, sql::proof::VerifiableQueryResult};
    let alloc = Bump::new();
    let tables = cats_table(&alloc);
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let mut accessor =
        TableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    for (table_ref, table) in &tables {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }
    let config = ConfigOptions::default();
    let statements = Parser::parse_sql(&GenericDialect {}, sql).unwrap();
    let plans = sql_to_proof_plans(&statements, &accessor, &config).unwrap();
    assert_eq!(plans.len(), expected_results.len());
    for (plan, expected) in plans.iter().zip(expected_results.iter()) {
        let res = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
            plan,
            &accessor,
            &&prover_setup,
            &[],
        )
        .unwrap();
        let res = res
            .verify(plan, &accessor, &&verifier_setup, &[])
            .unwrap()
            .table;
        assert_eq!(res, expected.clone());
    }
}

/// Searched CASE with numeric branches proves and verifies.
#[test]
fn test_case_when_numeric_branches() {
    use proof_of_sql::base::database::owned_table_utility::*;
    let sql = "SELECT CASE WHEN adopted THEN 1 ELSE 0 END AS flag FROM cats;
    SELECT CASE WHEN weight > 5.0 THEN 1 ELSE 0 END AS is_big FROM cats;
    SELECT CASE WHEN weight > 5.0 THEN id ELSE human_id END AS pick FROM cats;
    SELECT CASE WHEN id = 1 THEN 10 WHEN id = 2 THEN 20 ELSE 0 END AS v FROM cats;
    SELECT CASE human_id WHEN 1 THEN 10 ELSE 20 END AS v FROM cats;
    SELECT CASE WHEN adopted THEN CASE WHEN weight > 5.0 THEN 2 ELSE 1 END ELSE 0 END AS v FROM cats;";
    let expected_results = vec![
        owned_table([bigint("flag", [1_i64, 1, 0, 0, 1])]),
        // weights: 14.5, 7.5, 2.0, 4.5, 5.5
        owned_table([bigint("is_big", [1_i64, 1, 0, 0, 1])]),
        owned_table([int("pick", [1, 2, 1, 2, 5])]),
        // multiple arms with first-match-wins guards
        owned_table([bigint("v", [10_i64, 20, 0, 0, 0])]),
        // simple form CASE x WHEN v THEN ...
        owned_table([bigint("v", [10_i64, 10, 10, 20, 20])]),
        // nested CASE
        owned_table([bigint("v", [2_i64, 2, 0, 0, 2])]),
    ];
    assert_case_queries_prove_and_verify(sql, &expected_results);
}

/// Conditional aggregation - the highest-value CASE pattern.
#[test]
fn test_case_when_conditional_aggregation() {
    use proof_of_sql::base::database::owned_table_utility::*;
    let sql = "SELECT SUM(CASE WHEN adopted THEN 1 ELSE 0 END) AS adopted_count FROM cats;
    SELECT human_id, SUM(CASE WHEN adopted THEN weight ELSE 0.0 END) AS adopted_weight FROM cats GROUP BY human_id;";
    let expected_results = vec![
        owned_table([bigint("adopted_count", [3_i64])]),
        // human 1: 14.5 + 7.5 = 22.0; human 2: 5.5 (branches coerced to decimal(30,15))
        owned_table([
            int("human_id", [1, 2]),
            decimal75(
                "adopted_weight",
                30,
                15,
                [22_000_000_000_000_000_i128, 5_500_000_000_000_000],
            ),
        ]),
    ];
    assert_case_queries_prove_and_verify(sql, &expected_results);
}

/// CASE flavors that cannot be represented are rejected at planning time.
#[test]
fn test_unsupported_case_when_variants() {
    use proof_of_sql_planner::PlannerError;
    let alloc = Bump::new();
    let tables = cats_table(&alloc);
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let mut accessor =
        TableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    for (table_ref, table) in &tables {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }
    let config = ConfigOptions::default();

    // Varchar branches: strings cannot be rebuilt from the masked hash sum.
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT CASE WHEN weight > 5.0 THEN 'big' ELSE 'small' END FROM cats",
    )
    .unwrap();
    assert!(matches!(
        sql_to_proof_plans(&statements, &accessor, &config),
        Err(PlannerError::AnalyzeError { .. })
    ));

    // Missing ELSE: unmatched rows would be NULL, which has no representation.
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT CASE WHEN adopted THEN 1 END FROM cats",
    )
    .unwrap();
    assert!(matches!(
        sql_to_proof_plans(&statements, &accessor, &config),
        Err(PlannerError::UnsupportedLogicalExpression { .. })
    ));
}

/// Probe which CASE WHEN flavors convert to proof plans today.
/// Run with: cargo test -p proof-of-sql-planner --test case_when_matrix_tests probe -- --nocapture --ignored
#[test]
#[ignore = "diagnostic probe, prints plans; not a regression test"]
fn probe_case_when_feasibility() {
    let alloc = Bump::new();
    let tables = cats_table(&alloc);
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let mut accessor =
        TableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    for (table_ref, table) in &tables {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }
    let config = ConfigOptions::default();

    let queries = [
        // searched CASE, numeric branches
        "SELECT CASE WHEN adopted THEN 1 ELSE 0 END FROM cats",
        "SELECT CASE WHEN weight > 5.0 THEN 1 ELSE 0 END FROM cats",
        "SELECT CASE WHEN weight > 5.0 THEN id ELSE human_id END FROM cats",
        // multiple WHEN arms
        "SELECT CASE WHEN id = 1 THEN 10 WHEN id = 2 THEN 20 ELSE 0 END FROM cats",
        // varchar branches
        "SELECT CASE WHEN weight > 5.0 THEN 'big' ELSE 'small' END FROM cats",
        // simple CASE form (CASE expr WHEN value ...)
        "SELECT CASE human_id WHEN 1 THEN 10 ELSE 20 END FROM cats",
        // no ELSE (implicit NULL)
        "SELECT CASE WHEN adopted THEN 1 END FROM cats",
        // CASE in WHERE
        "SELECT id FROM cats WHERE CASE WHEN human_id = 1 THEN adopted ELSE true END",
        // conditional aggregation
        "SELECT SUM(CASE WHEN adopted THEN 1 ELSE 0 END) FROM cats",
        "SELECT human_id, SUM(CASE WHEN adopted THEN weight ELSE 0.0 END) FROM cats GROUP BY human_id",
        // nested CASE
        "SELECT CASE WHEN adopted THEN CASE WHEN weight > 5.0 THEN 2 ELSE 1 END ELSE 0 END FROM cats",
        // constant-foldable CASE
        "SELECT CASE WHEN 1 = 1 THEN id ELSE human_id END FROM cats",
    ];
    for sql in queries {
        let statements = match Parser::parse_sql(&GenericDialect {}, sql) {
            Ok(s) => s,
            Err(e) => {
                println!("=== PARSE-FAIL: {sql}\n--- error: {e}");
                continue;
            }
        };
        match sql_to_proof_plans(&statements, &accessor, &config) {
            Ok(plans) => {
                println!("=== OK: {sql}");
                println!("--- plan:\n{:#?}", plans[0]);
            }
            Err(e) => {
                println!("=== UNSUPPORTED: {sql}\n--- error: {e}");
            }
        }
    }
}
