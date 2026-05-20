use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table, TableRef},
        map::{indexmap, IndexSet},
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::{
        proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
        proof_exprs::{AndExpr, ColumnExpr, DynProofExpr, NotExpr, OrExpr, ProofExpr},
        AnalyzeError,
    },
};
use alloc::{boxed::Box, collections::VecDeque, vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;

fn column_ref(name: &str, column_type: ColumnType) -> ColumnRef {
    ColumnRef::new(TableRef::new("sxt", "t"), Ident::from(name), column_type)
}

#[test]
fn we_can_create_and_inspect_an_and_expr_without_blitzar() {
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);

    let expr = AndExpr::try_new(
        Box::new(DynProofExpr::new_column(lhs_ref.clone())),
        Box::new(DynProofExpr::new_column(rhs_ref.clone())),
    )
    .unwrap();

    assert_eq!(expr.data_type(), ColumnType::Boolean);
    assert_eq!(expr.lhs().data_type(), ColumnType::Boolean);
    assert_eq!(expr.rhs().data_type(), ColumnType::Boolean);

    let mut columns = IndexSet::default();
    expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 2);
    assert!(columns.contains(&lhs_ref));
    assert!(columns.contains(&rhs_ref));
}

#[test]
fn we_cannot_create_an_and_expr_from_mismatched_types_without_blitzar() {
    let err = AndExpr::try_new(
        Box::new(DynProofExpr::new_column(column_ref(
            "lhs",
            ColumnType::Boolean,
        ))),
        Box::new(DynProofExpr::new_column(column_ref(
            "rhs",
            ColumnType::BigInt,
        ))),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}

#[test]
fn we_can_evaluate_and_expr_without_blitzar() {
    let alloc = Bump::new();
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        lhs_ref.column_id() => Column::Boolean::<TestScalar>(lhs),
        rhs_ref.column_id() => Column::Boolean::<TestScalar>(rhs),
    })
    .unwrap();
    let expr = AndExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref))),
    )
    .unwrap();

    let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
    assert_eq!(first_round, Column::Boolean(&[true, false, false, false]));

    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let final_round = expr
        .final_round_evaluate(&mut builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(final_round, Column::Boolean(&[true, false, false, false]));
    assert_eq!(builder.pcs_proof_mles().len(), 1);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
}

#[test]
fn we_can_verify_an_and_expr_without_blitzar() {
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);
    let expr = AndExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone()))),
    )
    .unwrap();
    let accessor = indexmap! {
        lhs_ref.column_id() => TestScalar::ONE,
        rhs_ref.column_id() => TestScalar::ZERO,
    };
    let mut builder = MockVerificationBuilder::new(
        vec![],
        3,
        vec![],
        vec![vec![TestScalar::ZERO]],
        vec![],
        vec![],
        vec![],
    );

    let eval = expr
        .verifier_evaluate(&mut builder, &accessor, TestScalar::ONE, &[])
        .unwrap();

    assert_eq!(eval, TestScalar::ZERO);
    assert_eq!(builder.get_identity_results(), vec![vec![true]]);
}

#[test]
fn we_can_create_and_inspect_an_or_expr_without_blitzar() {
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);

    let expr = OrExpr::try_new(
        Box::new(DynProofExpr::new_column(lhs_ref.clone())),
        Box::new(DynProofExpr::new_column(rhs_ref.clone())),
    )
    .unwrap();

    assert_eq!(expr.data_type(), ColumnType::Boolean);
    assert_eq!(expr.lhs().data_type(), ColumnType::Boolean);
    assert_eq!(expr.rhs().data_type(), ColumnType::Boolean);

    let mut columns = IndexSet::default();
    expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 2);
    assert!(columns.contains(&lhs_ref));
    assert!(columns.contains(&rhs_ref));
}

#[test]
fn we_cannot_create_an_or_expr_from_mismatched_types_without_blitzar() {
    let err = OrExpr::try_new(
        Box::new(DynProofExpr::new_column(column_ref(
            "lhs",
            ColumnType::Boolean,
        ))),
        Box::new(DynProofExpr::new_column(column_ref(
            "rhs",
            ColumnType::BigInt,
        ))),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}

#[test]
fn we_can_evaluate_or_expr_without_blitzar() {
    let alloc = Bump::new();
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        lhs_ref.column_id() => Column::Boolean::<TestScalar>(lhs),
        rhs_ref.column_id() => Column::Boolean::<TestScalar>(rhs),
    })
    .unwrap();
    let expr = OrExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref))),
    )
    .unwrap();

    let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
    assert_eq!(first_round, Column::Boolean(&[true, true, true, false]));

    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let final_round = expr
        .final_round_evaluate(&mut builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(final_round, Column::Boolean(&[true, true, true, false]));
    assert_eq!(builder.pcs_proof_mles().len(), 1);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
}

#[test]
fn we_can_verify_an_or_expr_without_blitzar() {
    let lhs_ref = column_ref("lhs", ColumnType::Boolean);
    let rhs_ref = column_ref("rhs", ColumnType::Boolean);
    let expr = OrExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone()))),
    )
    .unwrap();
    let accessor = indexmap! {
        lhs_ref.column_id() => TestScalar::ONE,
        rhs_ref.column_id() => TestScalar::ZERO,
    };
    let mut builder = MockVerificationBuilder::new(
        vec![],
        3,
        vec![],
        vec![vec![TestScalar::ZERO]],
        vec![],
        vec![],
        vec![],
    );

    let eval = expr
        .verifier_evaluate(&mut builder, &accessor, TestScalar::ONE, &[])
        .unwrap();

    assert_eq!(eval, TestScalar::ONE);
    assert_eq!(builder.get_identity_results(), vec![vec![true]]);
}

#[test]
fn we_can_create_and_inspect_a_not_expr_without_blitzar() {
    let input_ref = column_ref("input", ColumnType::Boolean);

    let expr = NotExpr::try_new(Box::new(DynProofExpr::new_column(input_ref.clone()))).unwrap();

    assert_eq!(expr.data_type(), ColumnType::Boolean);
    assert_eq!(expr.input().data_type(), ColumnType::Boolean);

    let mut columns = IndexSet::default();
    expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 1);
    assert!(columns.contains(&input_ref));
}

#[test]
fn we_cannot_create_a_not_expr_from_a_non_boolean_type_without_blitzar() {
    let err = NotExpr::try_new(Box::new(DynProofExpr::new_column(column_ref(
        "input",
        ColumnType::BigInt,
    ))))
    .unwrap_err();

    assert!(matches!(
        err,
        AnalyzeError::InvalidDataType { expr_type: _ }
    ));
}

#[test]
fn we_can_evaluate_not_expr_without_blitzar() {
    let alloc = Bump::new();
    let input_ref = column_ref("input", ColumnType::Boolean);
    let input = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        input_ref.column_id() => Column::Boolean::<TestScalar>(input),
    })
    .unwrap();
    let expr =
        NotExpr::try_new(Box::new(DynProofExpr::Column(ColumnExpr::new(input_ref)))).unwrap();

    let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
    assert_eq!(first_round, Column::Boolean(&[false, true, false, true]));

    let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
    let final_round = expr
        .final_round_evaluate(&mut builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(final_round, Column::Boolean(&[false, true, false, true]));
    assert_eq!(builder.pcs_proof_mles().len(), 0);
    assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
}

#[test]
fn we_can_verify_a_not_expr_without_blitzar() {
    let input_ref = column_ref("input", ColumnType::Boolean);
    let expr = NotExpr::try_new(Box::new(DynProofExpr::Column(ColumnExpr::new(
        input_ref.clone(),
    ))))
    .unwrap();
    let accessor = indexmap! {
        input_ref.column_id() => TestScalar::ONE,
    };
    let mut builder =
        MockVerificationBuilder::new(vec![], 0, vec![], vec![], vec![], vec![], vec![]);

    let eval = expr
        .verifier_evaluate(&mut builder, &accessor, TestScalar::ONE, &[])
        .unwrap();

    assert_eq!(eval, TestScalar::ZERO);
}
