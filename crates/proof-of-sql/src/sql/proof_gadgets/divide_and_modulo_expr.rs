use crate::{
    base::{
        database::{Column, ColumnRef, Table},
        map::IndexMap,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::{divide_columns, modulo_columns, DynProofExpr, ProofExpr},
    },
    utils::log,
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// TODO: This struct is only partially complete. This should not be used yet. Several constraints still need to be added.
/// A gadget for proving divide and modulo expressions in tandem.
/// They must be proved in tandem under this protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivideAndModuloExpr {
    pub lhs: Box<DynProofExpr>,
    pub rhs: Box<DynProofExpr>,
}

trait DivideAndModuloExprUtilities<S: Scalar> {
    fn divide_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> (Column<'a, S>, &'a [S]);

    fn modulo_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> Column<'a, S>;
}

struct StandardDivideAndModuloExprUtilities;

impl<S: Scalar> DivideAndModuloExprUtilities<S> for StandardDivideAndModuloExprUtilities {
    fn divide_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> (Column<'a, S>, &'a [S]) {
        divide_columns(lhs, rhs, alloc)
    }

    fn modulo_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> Column<'a, S> {
        modulo_columns(lhs, rhs, alloc)
    }
}

impl DivideAndModuloExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }

    /// This is abstracted into its own function for ease of unit testing.
    /// The `utilities` function is where any functionality that needs to be mocked
    /// can be provided.
    fn prover_evaluate_base<'a, S: Scalar, U: DivideAndModuloExprUtilities<S>>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        utilities: &U,
    ) -> (Column<'a, S>, Column<'a, S>) {
        let lhs_column: Column<'a, S> = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.prover_evaluate(builder, alloc, table);

        let (quotient_wrapped, quotient) =
            utilities.divide_columns(&lhs_column, &rhs_column, alloc);
        let remainder = utilities.modulo_columns(&lhs_column, &rhs_column, alloc);
        builder.produce_intermediate_mle(quotient_wrapped);
        builder.produce_intermediate_mle(quotient);
        builder.produce_intermediate_mle(remainder);

        // subpolynomial: q * b + r - a = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(quotient), Box::new(rhs_column)]),
                (S::one(), vec![Box::new(remainder)]),
                (-S::one(), vec![Box::new(lhs_column)]),
            ],
        );

        (quotient_wrapped, remainder)
    }

    #[cfg_attr(not(test), expect(dead_code))]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> (Column<'a, S>, Column<'a, S>) {
        log::log_memory_usage("Start");
        let utilities = StandardDivideAndModuloExprUtilities {};

        let res = self.prover_evaluate_base(builder, alloc, table, &utilities);

        log::log_memory_usage("End");

        res
    }

    #[cfg_attr(not(test), expect(dead_code))]
    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        builder: &mut B,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<(S, S), ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor, one_eval)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor, one_eval)?;

        // lhs_times_rhs
        let quotient_wrapped = builder.try_consume_final_round_mle_evaluation()?;
        let quotient = builder.try_consume_final_round_mle_evaluation()?;
        let remainder = builder.try_consume_final_round_mle_evaluation()?;

        // subpolynomial: q * b - a + r = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            quotient * rhs - lhs + remainder,
            2,
        )?;

        Ok((quotient_wrapped, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DivideAndModuloExpr, DivideAndModuloExprUtilities, StandardDivideAndModuloExprUtilities,
    };
    use crate::{
        base::{
            database::{Column, ColumnRef, ColumnType, Table, TableRef},
            map::indexmap,
            polynomial::MultilinearExtension,
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::{mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder},
            proof_exprs::{columns_to_scalar_slice, ColumnExpr, DynProofExpr},
        },
    };
    use bumpalo::Bump;
    use mockall::automock;
    use sqlparser::ast::Ident;
    use std::collections::VecDeque;

    #[automock]
    trait MockableDivideAndModuloExprFunctionality {
        fn divide_columns(&self, lhs: &[i128], rhs: &[i128]) -> (Vec<TestScalar>, Vec<TestScalar>);

        fn modulo_columns(&self, lhs: &[i128], rhs: &[i128]) -> Vec<i128>;
    }

    struct MockDivideAndModuloExprUtilities<F: MockableDivideAndModuloExprFunctionality> {
        functions: F,
    }

    impl<F: MockableDivideAndModuloExprFunctionality> DivideAndModuloExprUtilities<TestScalar>
        for MockDivideAndModuloExprUtilities<F>
    {
        fn divide_columns<'a>(
            &self,
            lhs: &Column<'a, TestScalar>,
            rhs: &Column<'a, TestScalar>,
            alloc: &'a Bump,
        ) -> (Column<'a, TestScalar>, &'a [TestScalar]) {
            if let (Column::Int128(a), Column::Int128(b)) = (lhs, rhs) {
                let (quotient_wrapped, quotient) = self.functions.divide_columns(a, b);
                let quotient_wrapped_slice = alloc.alloc_slice_copy(&quotient_wrapped);
                let quotient_slice = alloc.alloc_slice_copy(&quotient);
                (Column::Scalar(quotient_wrapped_slice), quotient_slice)
            } else {
                panic!("MockDivideAndModuloExprUtilities should only be used with int128 columns");
            }
        }

        fn modulo_columns<'a>(
            &self,
            lhs: &Column<'a, TestScalar>,
            rhs: &Column<'a, TestScalar>,
            alloc: &'a Bump,
        ) -> Column<'a, TestScalar> {
            if let (Column::Int128(a), Column::Int128(b)) = (lhs, rhs) {
                let remainder = self.functions.modulo_columns(a, b);
                let remainder_slice = alloc.alloc_slice_copy(&remainder);
                Column::Int128(remainder_slice)
            } else {
                panic!("MockDivideAndModuloExprUtilities should only be used with int128 columns");
            }
        }
    }

    fn default_divide_columns(lhs: &[i128], rhs: &[i128]) -> (Vec<TestScalar>, Vec<TestScalar>) {
        let alloc = Bump::new();
        let standard_utilities = StandardDivideAndModuloExprUtilities;
        let (quotient_wrapped, quotient) = standard_utilities.divide_columns(
            &Column::Int128::<TestScalar>(lhs),
            &Column::Int128(rhs),
            &alloc,
        );
        (
            columns_to_scalar_slice(&quotient_wrapped, &alloc).to_vec(),
            quotient.to_vec(),
        )
    }

    fn default_modulo_columns(lhs: &[i128], rhs: &[i128]) -> Vec<i128> {
        let alloc = Bump::new();
        let standard_utilities = StandardDivideAndModuloExprUtilities;
        standard_utilities
            .modulo_columns(
                &Column::Int128::<TestScalar>(lhs),
                &Column::Int128(rhs),
                &alloc,
            )
            .as_int128()
            .unwrap()
            .to_vec()
    }

    #[derive(PartialEq, Debug)]
    enum TestableConstraints {
        /// q * b + r - a = 0
        DivisionAlgorithm,
    }

    fn get_failing_constraints(row: &[bool]) -> Vec<TestableConstraints> {
        assert_eq!(row.len(), 1);
        row.iter()
            .filter_map(|include| {
                if *include {
                    None
                } else {
                    Some(TestableConstraints::DivisionAlgorithm)
                }
            })
            .collect()
    }

    fn check_constraints(
        divide_columns_return: Option<(Vec<TestScalar>, Vec<TestScalar>)>,
        modulo_columns_return: Option<Vec<i128>>,
        lhs: &[i128],
        rhs: &[i128],
        expected_failing_constraints: &[TestableConstraints],
    ) {
        let alloc = Bump::new();
        let mut mock_functionality = MockMockableDivideAndModuloExprFunctionality::new();
        if let Some(quotient) = divide_columns_return {
            mock_functionality
                .expect_divide_columns()
                .return_const(quotient);
        } else {
            mock_functionality
                .expect_divide_columns()
                .returning(default_divide_columns);
        }
        if let Some(remainder) = modulo_columns_return {
            mock_functionality
                .expect_modulo_columns()
                .return_const(remainder);
        } else {
            mock_functionality
                .expect_modulo_columns()
                .returning(default_modulo_columns);
        }
        let mock_utilities = MockDivideAndModuloExprUtilities {
            functions: mock_functionality,
        };
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let lhs_ident = Ident::from("lhs");
        let rhs_ident = Ident::from("rhs");
        let lhs_ref = ColumnRef::new(table_ref.clone(), lhs_ident.clone(), ColumnType::Int128);
        let rhs_ref = ColumnRef::new(table_ref, rhs_ident.clone(), ColumnType::Int128);
        let divide_and_modulo_expr = DivideAndModuloExpr::new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone()))),
            Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone()))),
        );
        let mut final_round_builder = FinalRoundBuilder::new(lhs.len(), VecDeque::new());
        let table = Table::try_new(indexmap! {
            lhs_ident => Column::Int128::<TestScalar>(lhs),
            rhs_ident => Column::Int128::<TestScalar>(rhs),
        })
        .unwrap();
        divide_and_modulo_expr.prover_evaluate_base(
            &mut final_round_builder,
            &alloc,
            &table,
            &mock_utilities,
        );
        let mock_verification_builder = run_verify_for_each_row(
            lhs.len(),
            &final_round_builder,
            4,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone() => lhs.inner_product(evaluation_point),
                    rhs_ref.clone() => rhs.inner_product(evaluation_point)
                };
                divide_and_modulo_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval)
                    .unwrap();
            },
        );
        let matrix = mock_verification_builder.get_identity_results();
        for row in matrix {
            let failing_constraints = get_failing_constraints(&row);
            assert_eq!(failing_constraints, expected_failing_constraints);
        }
    }

    #[test]
    fn we_can_verify_simple_expr() {
        let alloc = Bump::new();
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let lhs_ident = Ident::from("lhs");
        let rhs_ident = Ident::from("rhs");
        let lhs_ref = ColumnRef::new(table_ref.clone(), lhs_ident.clone(), ColumnType::Int128);
        let rhs_ref = ColumnRef::new(table_ref, rhs_ident.clone(), ColumnType::Int128);
        let divide_and_modulo_expr = DivideAndModuloExpr::new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone()))),
            Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone()))),
        );
        let lhs = &[i128::MAX, i128::MIN, 2];
        let rhs = &[3i128, 3, -4];
        let mut final_round_builder = FinalRoundBuilder::new(lhs.len(), VecDeque::new());
        let table = Table::try_new(indexmap! {
            lhs_ident => Column::Int128::<TestScalar>(lhs),
            rhs_ident => Column::Int128::<TestScalar>(rhs),
        })
        .unwrap();
        divide_and_modulo_expr.prover_evaluate(&mut final_round_builder, &alloc, &table);
        let mock_verification_builder = run_verify_for_each_row(
            lhs.len(),
            &final_round_builder,
            4,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone() => lhs.inner_product(evaluation_point),
                    rhs_ref.clone() => rhs.inner_product(evaluation_point)
                };
                divide_and_modulo_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval)
                    .unwrap();
            },
        );
        let matrix = mock_verification_builder.get_identity_results();
        assert!(matrix.into_iter().all(|v| v.into_iter().all(|b| b)));
    }

    /// Shifting remainder by a very small amount will fail the division algorithm
    #[test]
    fn we_can_reject_if_division_algorithm_fails() {
        check_constraints(
            None,
            Some(vec![1i128, -4, 1, -1]),
            &[8i128, -12, 8, -8],
            &[3i128, 7, -3, -3],
            &[TestableConstraints::DivisionAlgorithm],
        );
    }

    #[should_panic(
        expected = "MockDivideAndModuloExprUtilities should only be used with int128 columns"
    )]
    #[test]
    fn we_currently_cannot_use_anything_other_than_i128_for_mocking_divide() {
        let alloc = Bump::new();
        let mock_functionality = MockMockableDivideAndModuloExprFunctionality::new();
        let mock_utilities = MockDivideAndModuloExprUtilities {
            functions: mock_functionality,
        };
        mock_utilities.divide_columns(&Column::BigInt(&[]), &Column::BigInt(&[]), &alloc);
    }

    #[should_panic(
        expected = "MockDivideAndModuloExprUtilities should only be used with int128 columns"
    )]
    #[test]
    fn we_currently_cannot_use_anything_other_than_i128_for_mocking_modulo() {
        let alloc = Bump::new();
        let mock_functionality = MockMockableDivideAndModuloExprFunctionality::new();
        let mock_utilities = MockDivideAndModuloExprUtilities {
            functions: mock_functionality,
        };
        mock_utilities.modulo_columns(&Column::BigInt(&[]), &Column::BigInt(&[]), &alloc);
    }
}
