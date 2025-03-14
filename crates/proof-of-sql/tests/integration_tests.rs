//! Other integration tests for the proof-of-sql crate.
#![cfg(feature = "test")]
#![cfg_attr(test, allow(clippy::missing_panics_doc))]
use ark_std::test_rng;
#[cfg(feature = "blitzar")]
use proof_of_sql::base::commitment::InnerProductProof;
#[cfg(feature = "hyperkzg")]
use proof_of_sql::proof_primitive::hyperkzg::HyperKZGCommitmentEvaluationProof;
use proof_of_sql::{
    base::database::{
        owned_table_utility::*, OwnedTable, OwnedTableTestAccessor, TableRef, TestAccessor,
    },
    proof_primitive::{
        dory::{
            DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
            DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
        },
        inner_product::curve_25519_scalar::Curve25519Scalar,
    },
    sql::{
        parse::{ConversionError, QueryExpr},
        postprocessing::apply_postprocessing_steps,
        proof::{QueryError, VerifiableQueryResult},
        AnalyzeError,
    },
};

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_minimal_filter_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([boolean("a", [true, false])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a;".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([boolean("a", [true])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_minimal_filter_query_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([boolean("a", [true, false])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE not a".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([boolean("a", [false])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_minimal_filter_query_with_dynamic_dory() {
    let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([boolean("a", [true, false])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE not a".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &&prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &&verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([boolean("a", [false])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_equality_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 1])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b = 1;".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [1, 3]), bigint("b", [1, 1])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_equality_query_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 1])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b = 1".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [1, 3]), bigint("b", [1, 1])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "hyperkzg")]
fn we_can_prove_a_basic_equality_query_with_hyperkzg() {
    use nova_snark::{
        provider::hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
        traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
    };
    type CP = HyperKZGCommitmentEvaluationProof;

    let ck: CommitmentKey<_> = CommitmentEngine::setup(b"test", 32);
    let (_, vk) = EvaluationEngine::setup(&ck);

    let mut accessor = OwnedTableTestAccessor::<CP>::new_empty_with_setup(&ck);
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 1])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b = 1".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<CP>::new(query.proof_expr(), &accessor, &&ck);
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &&vk)
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [1, 3]), bigint("b", [1, 1])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_inequality_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 2])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b >= 1;".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [1, 3]), bigint("b", [1, 2])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_query_containing_extrema_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            tinyint("tinyint", [i8::MIN, 0, i8::MAX]),
            smallint("smallint", [i16::MIN, 0, i16::MAX]),
            int("int", [i32::MIN, 0, i32::MAX]),
            bigint("bigint", [i64::MIN, 0, i64::MAX]),
            int128("int128", [i128::MIN, 0, i128::MAX]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        tinyint("tinyint", [i8::MIN, 0, i8::MAX]),
        smallint("smallint", [i16::MIN, 0, i16::MAX]),
        int("int", [i32::MIN, 0, i32::MAX]),
        bigint("bigint", [i64::MIN, 0, i64::MAX]),
        int128("int128", [i128::MIN, 0, i128::MAX]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_query_containing_extrema_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);
    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            tinyint("tinyint", [i8::MIN, 0, i8::MAX]),
            smallint("smallint", [i16::MIN, 0, i16::MAX]),
            int("int", [i32::MIN, 0, i32::MAX]),
            bigint("bigint", [i64::MIN, 0, i64::MAX]),
            int128("int128", [i128::MIN, 0, i128::MAX]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table;".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        tinyint("tinyint", [i8::MIN, 0, i8::MAX]),
        smallint("smallint", [i16::MIN, 0, i16::MAX]),
        int("int", [i32::MIN, 0, i32::MAX]),
        bigint("bigint", [i64::MIN, 0, i64::MAX]),
        int128("int128", [i128::MIN, 0, i128::MAX]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_query_with_arithmetic_in_where_clause_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [4, 1, 2])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b >= a + 1".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let transformed_result: OwnedTable<Curve25519Scalar> =
        apply_postprocessing_steps(owned_table_result, query.postprocessing()).unwrap();
    let expected_result = owned_table([bigint("a", [1]), bigint("b", [4])]);
    assert_eq!(transformed_result, expected_result);
}

#[test]
fn we_can_prove_a_query_with_arithmetic_in_where_clause_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);
    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, -1, 3]), bigint("b", [0, 0, 2])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b > 1 - a;".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [3]), bigint("b", [2])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_equality_with_out_of_order_results_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("public", "test_table"),
        owned_table([
            int128("amount", [115, -79]),
            varchar("primes", ["-f34", "abcd"]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "select primes, amount from public.test_table where primes = 'abcd';"
            .parse()
            .unwrap(),
        "public".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let transformed_result: OwnedTable<Curve25519Scalar> =
        apply_postprocessing_steps(owned_table_result, query.postprocessing()).unwrap();
    let expected_result = owned_table([varchar("primes", ["abcd"]), int128("amount", [-79])]);
    assert_eq!(transformed_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_inequality_query_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 4])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b <= 0".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [2]), bigint("b", [0])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn decimal_type_issues_should_cause_provable_ast_to_fail() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([decimal75("d0", 12, 0, [10])]),
        0,
    );
    let large_decimal = format!("0.{}", "1".repeat(75));
    let query_string = format!("SELECT d0 + {large_decimal} as res FROM table;");
    assert!(matches!(
        QueryExpr::try_new(query_string.parse().unwrap(), "sxt".into(), &accessor,),
        Err(ConversionError::AnalyzeError {
            source: AnalyzeError::DataTypeMismatch { .. }
        })
    ));
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_complex_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            smallint("a", [1_i16, 2, 3]),
            int("b", [1_i32, 4, 3]),
            bigint("c", [3_i64, 3, -3]),
            bigint("d", [1_i64, 2, 3]),
            varchar("e", ["d", "e", "f"]),
            boolean("f", [true, false, false]),
            decimal75("d0", 12, 4, [1, 2, 3]),
            decimal75("d1", 12, 2, [3, 4, 2]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a + (b * c) + 1 as t, 45.7 as g, (a = b) or f as h, d0 * d1 + 1.4 as dr FROM table WHERE (a >= b) = (c < d) and (e = 'e') = f;"
            .parse()
            .unwrap(),
            "sxt".into(),        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        bigint("t", [-5]),
        decimal75("g", 3, 1, [457]),
        boolean("h", [true]),
        decimal75("dr", 26, 6, [1_400_006]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_complex_query_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            smallint("a", [1_i16, 2, 3]),
            int("b", [1, 0, 1]),
            bigint("c", [3, 3, -3]),
            bigint("d", [1, 2, 3]),
            varchar("e", ["d", "e", "f"]),
            boolean("f", [true, false, true]),
            decimal75("d0", 12, 4, [1, 4, 3]),
            decimal75("d1", 12, 2, [3, 4, 2]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT 0.5 + a * b * c - d as res, 32 as g, (c >= d) and f as h, (a + 1) * (b + 1 + c + d + d0 - d1 + 0.5) as res2 FROM table WHERE (a < b) = (c <= d) and e <> 'f' and f and 100000 * d1 * d0 + a = 1.3"
            .parse()
            .unwrap(),
         "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        decimal75("res", 22, 1, [25]),
        bigint("g", [32]),
        boolean("h", [true]),
        decimal75("res2", 46, 4, [129_402]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

//TODO: This test uses postprocessing now. Check proof results once PROOF-765 is done.
#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_minimal_group_by_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [1, 1, 2, 2, 3]), bigint("b", [1, 0, 2, 3, 4])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, count(*) as c FROM table group by a"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result: OwnedTable<Curve25519Scalar> = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let transformed_result: OwnedTable<Curve25519Scalar> =
        apply_postprocessing_steps(owned_table_result, query.postprocessing()).unwrap();
    let expected_result: OwnedTable<Curve25519Scalar> =
        owned_table([bigint("a", [1_i64, 2, 3]), bigint("c", [2_i64, 2, 1])]);
    assert_eq!(transformed_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_basic_group_by_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            bigint("a", [1, 1, 2, 3, 2]),
            bigint("b", [1, 0, 4, 2, 3]),
            bigint("c", [-2, 2, 1, 0, 1]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, sum(2 * b + 1) as d, count(*) as e FROM table WHERE c >= 0 group by a"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        bigint("a", [1, 2, 3]),
        bigint("d", [1, 16, 5]),
        bigint("e", [1, 2, 1]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_cat_group_by_query_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "cats"),
        owned_table([
            int("id", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            varchar(
                "name",
                [
                    "Chloe",
                    "Margaret",
                    "Prudence",
                    "Lucy",
                    "Ms. Kitty",
                    "Pepper",
                    "Rocky",
                    "Smokey",
                    "Tiger",
                    "Whiskers",
                ],
            ),
            smallint("age", [12_i16, 2, 3, 3, 10, 2, 2, 4, 5, 6]),
            varchar(
                "human",
                [
                    "Ian", "Ian", "Gretta", "Gretta", "Gretta", "Gretta", "Gretta", "Alice", "Bob",
                    "Charlie",
                ],
            ),
            boolean(
                "is_female",
                [
                    true, true, true, true, true, true, false, false, false, false,
                ],
            ),
            bigint("proof_order", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "select human, sum(age + 0.1) as total_adjusted_cat_age, count(*) as num_cats from sxt.cats where is_female group by human order by human"
            .parse()
            .unwrap(),
            "sxt".into(),        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        varchar("human", ["Gretta", "Ian"]),
        decimal75("total_adjusted_cat_age", 7, 1, [184_i16, 142]),
        bigint("num_cats", [4, 2]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_cat_group_by_query_with_dynamic_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "cats"),
        owned_table([
            int("id", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            varchar(
                "name",
                [
                    "Chloe",
                    "Margaret",
                    "Prudence",
                    "Lucy",
                    "Ms. Kitty",
                    "Pepper",
                    "Rocky",
                    "Smokey",
                    "Tiger",
                    "Whiskers",
                ],
            ),
            decimal75(
                "diff_from_ideal_weight",
                3,
                1,
                [103_i16, -20, 34, 34, 103, -25, -25, 47, 52, 63],
            ),
            varchar(
                "human",
                [
                    "Ian", "Ian", "Gretta", "Gretta", "Gretta", "Gretta", "Gretta", "Alice", "Bob",
                    "Charlie",
                ],
            ),
            boolean(
                "is_female",
                [
                    true, true, true, true, true, true, false, false, false, false,
                ],
            ),
            bigint("proof_order", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "select diff_from_ideal_weight, count(*) as num_cats from sxt.cats where is_female group by diff_from_ideal_weight order by diff_from_ideal_weight"
            .parse()
            .unwrap(),
    "sxt".into(),        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &&prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &&verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        decimal75("diff_from_ideal_weight", 3, 1, [-25, -20, 34, 103]),
        bigint("num_cats", [1_i64, 1, 2, 2]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn we_can_prove_a_basic_group_by_query_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            bigint("a", [1, 1, 2, 3, 2]),
            bigint("b", [1, 0, 4, 2, 3]),
            bigint("c", [-2, 2, 1, 0, 1]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, sum(2 * b + 1) as d, count(*) as e FROM table WHERE c >= 0 group by a"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        bigint("a", [1, 2, 3]),
        bigint("d", [1, 16, 5]),
        bigint("e", [1, 2, 1]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_varbinary_equality_query_with_hex_literal() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            bigint("a", [123, 4567]),
            varbinary("b", [vec![1, 2, 3], vec![4, 5, 6, 7]]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a, b FROM table WHERE b = 0x04050607"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([bigint("a", [4567]), varbinary("b", [vec![4, 5, 6, 7]])]);
    assert_eq!(owned_table_result, expected_result);
}

// Overflow checks
#[test]
#[cfg(feature = "blitzar")]
fn we_can_prove_a_query_with_overflow_with_curve25519() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([smallint("a", [i16::MAX]), smallint("b", [1_i16])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a + b as c from table".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    assert!(matches!(
        verifiable_result.verify(query.proof_expr(), &accessor, &()),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn we_can_prove_a_query_with_overflow_with_dory() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([bigint("a", [i64::MIN]), smallint("b", [1_i16])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a - b as c from table".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    assert!(matches!(
        verifiable_result.verify(query.proof_expr(), &accessor, &dory_verifier_setup,),
        Err(QueryError::Overflow)
    ));
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_perform_arithmetic_and_conditional_operations_on_tinyint() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            tinyint("a", [3_i8, 5, 2, 1]),
            tinyint("b", [2_i8, 1, 3, 4]),
            tinyint("c", [1_i8, 4, 5, 2]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT a*b+b+c as result FROM table WHERE a>b OR c=4"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([tinyint("result", [9_i8, 10])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_perform_equality_checks_on_var_binary() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            varbinary("a", [vec![], vec![], vec![], vec![]]),
            varbinary("b", [vec![], vec![], vec![], vec![]]),
            varbinary("c", [vec![], vec![], vec![], vec![]]),
            varbinary("d", [vec![], vec![], vec![], vec![]]),
            varbinary("e", [vec![], vec![], vec![], vec![]]),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a=b".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        varbinary("a", [vec![], vec![], vec![], vec![]]),
        varbinary("b", [vec![], vec![], vec![], vec![]]),
        varbinary("c", [vec![], vec![], vec![], vec![]]),
        varbinary("d", [vec![], vec![], vec![], vec![]]),
        varbinary("e", [vec![], vec![], vec![], vec![]]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
#[expect(clippy::too_many_lines)]
fn we_can_perform_rich_equality_checks_on_var_binary() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            varbinary(
                "a",
                [
                    vec![],
                    b"\x01\x02\x03\x04\x05".to_vec(),
                    vec![0xFF; 31],
                    vec![0xAA, 0xBB],
                ],
            ),
            varbinary(
                "b",
                [
                    vec![],
                    b"\x01\x02\x03\x04\x05".to_vec(),
                    vec![0xFF; 31],
                    vec![0xAA, 0xBB],
                ],
            ),
            varbinary(
                "c",
                [
                    b"\xDE\xAD\xBE\xEF".to_vec(),
                    vec![],
                    vec![0xFF; 31],
                    b"\x01\x02\x03".to_vec(),
                ],
            ),
            varbinary(
                "d",
                [
                    vec![],
                    b"\xAB\xCD".to_vec(),
                    vec![0xEE; 31],
                    b"\xFE".to_vec(),
                ],
            ),
            varbinary(
                "e",
                [
                    b"\xAA".to_vec(),
                    b"\xAA\xBB\xCC".to_vec(),
                    vec![0xDD; 31],
                    vec![],
                ],
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a=b".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;
    let expected_result = owned_table([
        varbinary(
            "a",
            [
                vec![],
                b"\x01\x02\x03\x04\x05".to_vec(),
                vec![0xFF; 31],
                vec![0xAA, 0xBB],
            ],
        ),
        varbinary(
            "b",
            [
                vec![],
                b"\x01\x02\x03\x04\x05".to_vec(),
                vec![0xFF; 31],
                vec![0xAA, 0xBB],
            ],
        ),
        varbinary(
            "c",
            [
                b"\xDE\xAD\xBE\xEF".to_vec(),
                vec![],
                vec![0xFF; 31],
                b"\x01\x02\x03".to_vec(),
            ],
        ),
        varbinary(
            "d",
            [
                vec![],
                b"\xAB\xCD".to_vec(),
                vec![0xEE; 31],
                b"\xFE".to_vec(),
            ],
        ),
        varbinary(
            "e",
            [
                b"\xAA".to_vec(),
                b"\xAA\xBB\xCC".to_vec(),
                vec![0xDD; 31],
                vec![],
            ],
        ),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
#[expect(clippy::too_many_lines)]
fn we_can_perform_equality_checks_on_rich_var_binary_data() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    // We'll create multiple columns to have richer data,
    // including some rows where a != b.
    // Only rows where a = b will appear in the final result.
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            varbinary(
                "a",
                [
                    vec![],           // row1
                    vec![0x10, 0x11], // row2
                    vec![0xAB, 0xCD], // row3
                    vec![0x12],       // row4
                    vec![0x34],       // row5
                    vec![1, 2, 3],    // row6
                ],
            ),
            varbinary(
                "b",
                [
                    vec![],           // row1
                    vec![0x11],       // row2
                    vec![0xAB, 0xCD], // row3
                    vec![0x12],       // row4
                    vec![0x34],       // row5
                    vec![1, 2, 3],    // row6
                ],
            ),
            varbinary(
                "c",
                [
                    vec![0x00; 31],   // row1
                    vec![0x22],       // row2
                    vec![],           // row3
                    vec![0x98, 0x99], // row4
                    vec![0x56],       // row5
                    vec![4, 5],       // row6
                ],
            ),
            varbinary(
                "d",
                [
                    vec![0x00; 31],   // row1
                    vec![0x22],       // row2
                    vec![0x00],       // row3
                    vec![0x98, 0x99], // row4
                    vec![0x56],       // row5
                    vec![4, 5],       // row6
                ],
            ),
            varbinary(
                "e",
                [
                    vec![0xFF, 0x00], // row1
                    vec![0x33],       // row2
                    vec![0xDD; 31],   // row3
                    vec![0xFF; 31],   // row4
                    vec![0x78],       // row5
                    vec![6, 7],       // row6
                ],
            ),
            varbinary(
                "f",
                [
                    vec![0xFF, 0x00], // row1
                    vec![0x33],       // row2
                    vec![0xDD; 31],   // row3
                    vec![0xFF; 31],   // row4
                    vec![0x78],       // row5
                    vec![6, 7, 8],    // row6
                ],
            ),
            varbinary(
                "g",
                [
                    vec![0xA1], // row1
                    vec![0xA2], // row2
                    vec![0xA3], // row3
                    vec![0xA4], // row4
                    vec![0xA5], // row5
                    vec![0xA6], // row6
                ],
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a=b AND c=d AND e=f"
            .parse()
            .unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result =
        VerifiableQueryResult::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &())
        .unwrap()
        .table;

    let expected_result = owned_table([
        varbinary("a", [vec![], vec![0x12], vec![0x34]]),
        varbinary("b", [vec![], vec![0x12], vec![0x34]]),
        varbinary("c", [vec![0x00; 31], vec![0x98, 0x99], vec![0x56]]),
        varbinary("d", [vec![0x00; 31], vec![0x98, 0x99], vec![0x56]]),
        varbinary("e", [vec![0xFF, 0x00], vec![0xFF; 31], vec![0x78]]),
        varbinary("f", [vec![0xFF, 0x00], vec![0xFF; 31], vec![0x78]]),
        varbinary("g", [vec![0xA1], vec![0xA4], vec![0xA5]]),
    ]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_perform_equality_checks_on_fixed_size_binary() {
    use proof_of_sql::{
        base::{
            database::{owned_table_utility::*, TableRef},
            math::fixed_size_binary_width::FixedSizeBinaryWidth,
        },
        sql::{parse::QueryExpr, proof::VerifiableQueryResult},
    };
    use std::convert::TryFrom;
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);
    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        TableRef::new("sxt", "table"),
        owned_table([
            fixed_size_binary(
                "a",
                FixedSizeBinaryWidth::try_from(4).unwrap(),
                vec![0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4],
            ),
            fixed_size_binary(
                "b",
                FixedSizeBinaryWidth::try_from(4).unwrap(),
                vec![0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 9],
            ),
        ]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a=b".parse().unwrap(),
        "sxt".into(),
        &accessor,
    )
    .unwrap();
    let verifiable_result = VerifiableQueryResult::<DoryEvaluationProof>::new(
        query.proof_expr(),
        &accessor,
        &dory_prover_setup,
    );
    let owned_table_result = verifiable_result
        .verify(query.proof_expr(), &accessor, &dory_verifier_setup)
        .unwrap()
        .table;
    let expected_result = owned_table([
        fixed_size_binary(
            "a",
            FixedSizeBinaryWidth::try_from(4).unwrap(),
            vec![0, 0, 0, 0, 1, 2, 3, 4],
        ),
        fixed_size_binary(
            "b",
            FixedSizeBinaryWidth::try_from(4).unwrap(),
            vec![0, 0, 0, 0, 1, 2, 3, 4],
        ),
    ]);
    assert_eq!(owned_table_result, expected_result);
}
