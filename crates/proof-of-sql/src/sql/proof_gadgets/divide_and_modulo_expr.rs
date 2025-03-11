use super::{prover_evaluate_sign, verifier_evaluate_sign};
use crate::{
    base::{
        database::{try_divide_modulo_column_types, Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::{
            add_subtract_columns, columns_to_scalar_slice, divide_columns, modulo_columns,
            DynProofExpr, ProofExpr,
        },
    },
    utils::log,
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

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

    fn get_in_range_column_from_quotient_and_rhs<'a>(
        &self,
        alloc: &'a Bump,
        quotient: &'a [S],
        rhs: Vec<S>,
        min_sqrt_scalar: S,
    ) -> &'a [S];
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

    fn get_in_range_column_from_quotient_and_rhs<'a>(
        &self,
        alloc: &'a Bump,
        quotient: &'a [S],
        rhs: Vec<S>,
        min_sqrt_scalar: S,
    ) -> &'a [S] {
        let in_range_q_or_b = alloc.alloc_slice_fill_with(quotient.len(), |_i| S::ZERO);
        for (res, (q, b)) in in_range_q_or_b
            .iter_mut()
            .zip(quotient.iter().copied().zip(rhs.clone()))
        {
            // We do or rather than and here because scalars wrap negative values, so only one can be true at a time
            let in_range_value = if q < min_sqrt_scalar || q > -min_sqrt_scalar {
                q
            } else {
                b
            };
            *res = in_range_value;
        }
        in_range_q_or_b
    }
}

impl DivideAndModuloExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }

    #[allow(clippy::missing_panics_doc)]
    fn min_scalar<S: Scalar>(&self) -> S {
        self.lhs.data_type().min_scalar::<S>().unwrap()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn data_type(&self) -> (ColumnType, ColumnType) {
        try_divide_modulo_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to divide/modulo column types")
    }

    /// This is abstracted into its own function for ease of unit testing.
    /// The `utilities` function is where any functionality that needs to be mocked
    /// can be provided.
    /// # Panics
    /// Panics if the datatype is not supported
    #[expect(clippy::too_many_lines)]
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

        // (r - b) * (r + b) * t' - b = 0, where t' = b / ((r - b) * (r + b)) when |r| is not |b|
        // This confirms |r| = |b| only if b = 0.
        let remainder_minus_rhs = add_subtract_columns(remainder, rhs_column, 0, 0, alloc, true);
        let remainder_plus_rhs = add_subtract_columns(remainder, rhs_column, 0, 0, alloc, false);
        let rhs_as_scalars = rhs_column.to_scalar_with_scaling(0);
        let rhs_div_remainder_rhs_difference_of_squares =
            alloc.alloc_slice_fill_with(rhs_column.len(), |_i| S::ZERO);
        for (res, ((diff, add), b)) in rhs_div_remainder_rhs_difference_of_squares.iter_mut().zip(
            remainder_minus_rhs
                .iter()
                .copied()
                .zip(remainder_plus_rhs.iter().copied())
                .zip(rhs_as_scalars.clone()),
        ) {
            *res = (diff * add).inv().unwrap_or(S::ONE) * b;
        }
        let t = Column::Scalar(rhs_div_remainder_rhs_difference_of_squares);
        builder.produce_intermediate_mle(t);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![
                        Box::new(remainder_minus_rhs),
                        Box::new(remainder_plus_rhs),
                        Box::new(t),
                    ],
                ),
                (-S::one(), vec![Box::new(rhs_column)]),
            ],
        );

        // (s - q) * (s - b) = 0
        // Introduces a value s that must be either q or b.
        // We choose s to be a value of q or b such that -sqrt(-MIN) < s < sqrt(-MIN)
        let min_sqrt_scalar = S::from(self.lhs.data_type().sqrt_negative_min().unwrap());
        let in_range_q_or_b = utilities.get_in_range_column_from_quotient_and_rhs(
            alloc,
            quotient,
            rhs_as_scalars.clone(),
            min_sqrt_scalar,
        );
        let s = Column::Scalar(in_range_q_or_b);
        builder.produce_intermediate_mle(s);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(s), Box::new(s)]),
                (S::one(), vec![Box::new(rhs_column), Box::new(quotient)]),
                (-S::one(), vec![Box::new(s), Box::new(rhs_column)]),
                (-S::one(), vec![Box::new(s), Box::new(quotient)]),
            ],
        );

        // b * u = q where u = q / b if b is not 0
        // This ensures that q = 0 if b = 0
        let q_div_b = alloc.alloc_slice_fill_with(quotient.len(), |_i| S::ZERO);
        for (res, (q, b)) in q_div_b
            .iter_mut()
            .zip(quotient.iter().copied().zip(rhs_as_scalars))
        {
            *res = b.inv().unwrap_or(S::ONE) * q;
        }
        let u = Column::Scalar(q_div_b);
        builder.produce_intermediate_mle(u);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(rhs_column), Box::new(u)]),
                (-S::one(), vec![Box::new(quotient)]),
            ],
        );
        // (q′ − q) * (q + MIN) = 0
        // Ensures that either q = q' or q = -MIN
        // Simplifies to
        // q' * q - MIN * q - q * q + MIN * q'

        let min_scalar = self.min_scalar();
        let min_column =
            Column::Scalar(alloc.alloc_slice_fill_with(quotient.len(), |_i| min_scalar));

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![Box::new(quotient_wrapped), Box::new(quotient)],
                ),
                (-S::one(), vec![Box::new(min_column), Box::new(quotient)]),
                (-S::one(), vec![Box::new(quotient), Box::new(quotient)]),
                (
                    S::one(),
                    vec![Box::new(quotient_wrapped), Box::new(min_column)],
                ),
            ],
        );

        // (q' - MIN) * (q + MIN) * v - (q' - MIN) = 0 where v = 1 / (q + MIN) if q is not - MIN
        // Ensures q = -MIN only if q' = MIN
        let quotient_plus_min_inverse = alloc.alloc_slice_fill_with(quotient.len(), |_i| S::ZERO);
        for (res, q) in quotient_plus_min_inverse
            .iter_mut()
            .zip(quotient.iter().copied())
        {
            *res = (q + min_scalar).inv().unwrap_or(S::ONE);
        }
        let v = Column::Scalar(quotient_plus_min_inverse);
        builder.produce_intermediate_mle(v);

        let min_scalar_column =
            Column::Scalar(alloc.alloc_slice_fill_with(quotient.len(), |_i| min_scalar));

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![Box::new(quotient_wrapped), Box::new(quotient), Box::new(v)],
                ),
                (S::one(), vec![Box::new(min_scalar_column)]),
                (-S::one(), vec![Box::new(quotient_wrapped)]),
                (
                    -S::one(),
                    vec![
                        Box::new(min_scalar_column),
                        Box::new(min_scalar_column),
                        Box::new(v),
                    ],
                ),
                (
                    -S::one(),
                    vec![Box::new(min_scalar_column), Box::new(quotient), Box::new(v)],
                ),
                (
                    S::one(),
                    vec![
                        Box::new(min_scalar_column),
                        Box::new(quotient_wrapped),
                        Box::new(v),
                    ],
                ),
            ],
        );

        // sign(sqrt(-min) + s) = 1
        // sign(sqrt(-min) - s) = 1
        // These confirm that q * b does not wrap in the Scalar field. Either q or b must be smaller than sqrt(-min), which confines qb to less than the order of the field.
        let neg_min_sqrt_scalar_column =
            Column::Scalar(alloc.alloc_slice_fill_with(quotient.len(), |_i| min_sqrt_scalar));
        prover_evaluate_sign(
            builder,
            alloc,
            add_subtract_columns(neg_min_sqrt_scalar_column, s, 0, 0, alloc, false),
        );
        prover_evaluate_sign(
            builder,
            alloc,
            add_subtract_columns(neg_min_sqrt_scalar_column, s, 0, 0, alloc, true),
        );

        // sign<128>(q)
        // Confirms that q is not too big.
        prover_evaluate_sign(builder, alloc, quotient);

        // sign(a) * r = sign(r) * r and sign(r - b) * b + sign(r - b) - b = 0
        // constrains remainder to be in the correct range
        let lhs_sign =
            prover_evaluate_sign(builder, alloc, columns_to_scalar_slice(&lhs_column, alloc));
        let remainder_sign =
            prover_evaluate_sign(builder, alloc, columns_to_scalar_slice(&remainder, alloc));

        let remainder_minus_rhs_sign = prover_evaluate_sign(builder, alloc, remainder_minus_rhs);
        let remainder_plus_rhs_sign = prover_evaluate_sign(builder, alloc, remainder_plus_rhs);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_sign), Box::new(remainder)]),
                (
                    -S::one(),
                    vec![Box::new(remainder_sign), Box::new(remainder)],
                ),
            ],
        );

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![Box::new(remainder_minus_rhs_sign), Box::new(rhs_column)],
                ),
                (
                    S::one(),
                    vec![Box::new(remainder_plus_rhs_sign), Box::new(rhs_column)],
                ),
                (-S::one(), vec![Box::new(rhs_column)]),
            ],
        );

        (quotient_wrapped, remainder)
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn prover_evaluate<'a, S: Scalar>(
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

    /// # Panics
    /// Panics if the datatype is not supported
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
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

        // (r - b) * (r + b) * t' - b = 0
        let t = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (remainder - rhs) * (remainder + rhs) * t - rhs,
            3,
        )?;

        // (s - q)(s - b) = 0
        let s = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (s - quotient) * (s - rhs),
            2,
        )?;

        // b * u = q
        let q_div_b = builder.try_consume_final_round_mle_evaluation()?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            rhs * q_div_b - quotient,
            2,
        )?;

        // (q′ − q) * (q + MIN) = 0
        let min_eval = self.min_scalar::<S>() * one_eval;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (quotient_wrapped - quotient) * (quotient + min_eval),
            2,
        )?;

        // (q' - MIN) * (q + MIN) * v - (q' - MIN) = 0
        let v = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (quotient_wrapped - min_eval) * (quotient + min_eval) * v - quotient_wrapped + min_eval,
            3,
        )?;

        // get sign(sqrt(-min) + s)
        // get sign(sqrt(-min) - s)
        let min_sqrt_eval = S::from(self.data_type().0.sqrt_negative_min().unwrap()) * one_eval;
        let sqrt_min_plus_s = verifier_evaluate_sign(builder, min_sqrt_eval + s, one_eval, None)?;
        let sqrt_min_less_s = verifier_evaluate_sign(builder, min_sqrt_eval - s, one_eval, None)?;

        // MIN < q < -MIN
        // We need at least an extra bit to allow for -MIN
        verifier_evaluate_sign(
            builder,
            quotient,
            one_eval,
            Some(
                (self.data_type().0.to_integer_bits().unwrap() + 1)
                    .try_into()
                    .unwrap(),
            ),
        )?;

        // sign(a) * r = sign(r) * r and sign(r - b) * b + sign(r + b) * b = b
        let lhs_sign = verifier_evaluate_sign(builder, lhs, one_eval, None)?;
        let remainder_sign = verifier_evaluate_sign(builder, remainder, one_eval, None)?;

        let remainder_and_rhs_difference_sign =
            verifier_evaluate_sign(builder, remainder - rhs, one_eval, None)?;
        let remainder_and_rhs_added_sign =
            verifier_evaluate_sign(builder, remainder + rhs, one_eval, None)?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            remainder * (lhs_sign - remainder_sign),
            2,
        )?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            rhs * (remainder_and_rhs_difference_sign + remainder_and_rhs_added_sign - S::ONE),
            2,
        )?;

        // sign(sqrt(-min) + s) = 1
        // sign(sqrt(-min) - s) = 1
        if sqrt_min_plus_s != S::ZERO || sqrt_min_less_s != S::ZERO {
            return Err(ProofError::VerificationError {
                error: "Intermediate value out of range",
            });
        }

        Ok((quotient, remainder))
    }

    #[expect(dead_code)]
    pub fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
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
            proof::ProofError,
            scalar::{test_scalar::TestScalar, Scalar},
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

        fn get_in_range_column_from_quotient_and_rhs(
            &self,
            quotient: &[TestScalar],
            rhs: &[TestScalar],
        ) -> Vec<TestScalar>;
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

        fn get_in_range_column_from_quotient_and_rhs<'a>(
            &self,
            alloc: &'a Bump,
            quotient: &'a [TestScalar],
            rhs: Vec<TestScalar>,
            _min_sqrt_scalar: TestScalar,
        ) -> &'a [TestScalar] {
            alloc.alloc_slice_copy(
                &self
                    .functions
                    .get_in_range_column_from_quotient_and_rhs(quotient, &rhs),
            )
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

    fn default_get_in_range_column_from_quotient_and_rhs(
        quotient: &[TestScalar],
        rhs: &[TestScalar],
    ) -> Vec<TestScalar> {
        let alloc = Bump::new();
        let standard_utilities = StandardDivideAndModuloExprUtilities;
        standard_utilities
            .get_in_range_column_from_quotient_and_rhs(
                &alloc,
                quotient,
                rhs.to_vec(),
                TestScalar::from(ColumnType::Int128.sqrt_negative_min().unwrap()),
            )
            .to_vec()
    }

    #[derive(PartialEq, Debug)]
    enum TestableConstraints {
        /// q * b + r - a = 0
        DivisionAlgorithm,
        /// (r - b) * (r + b) * t' - b = 0
        DenominatorZeroIfRemainderAndDenominatorMagnitudeEqual,
        /// (s - q)(s - b) = 0
        BoundedValueIsQuotientOrDenominator,
        /// b * u - q = 0
        ZeroDenominatorDictatesQuotient,
        /// (q′ − q) * (q + MIN) = 0
        WrappedIsQuotientOrMin,
        /// (q' - MIN) * (q + MIN) * v - (q' - MIN) = 0
        WrappedIsMinIfQuotientIsNegativeMin,
        /// sign(a) * r - sign(r) * r = 0
        RemainderSignMatchesNumerator,
        /// sign(r - b) * b + sign(r + b) * b - b = 0
        RemainderBound,
    }

    fn get_failing_constraints(row: &[bool]) -> Vec<TestableConstraints> {
        assert_eq!(row.len(), 8);
        row.iter()
            .enumerate()
            .filter_map(|(i, include)| {
                if *include {
                    None
                } else {
                    Some(if i == 0 {
                        TestableConstraints::DivisionAlgorithm
                    } else if i == 1 {
                        TestableConstraints::DenominatorZeroIfRemainderAndDenominatorMagnitudeEqual
                    } else if i == 2 {
                        TestableConstraints::BoundedValueIsQuotientOrDenominator
                    } else if i == 3 {
                        TestableConstraints::ZeroDenominatorDictatesQuotient
                    } else if i == 4 {
                        TestableConstraints::WrappedIsQuotientOrMin
                    } else if i == 5 {
                        TestableConstraints::WrappedIsMinIfQuotientIsNegativeMin
                    } else if i == 6 {
                        TestableConstraints::RemainderSignMatchesNumerator
                    } else {
                        TestableConstraints::RemainderBound
                    })
                }
            })
            .collect()
    }

    fn check_constraints(
        divide_columns_return: Option<(Vec<TestScalar>, Vec<TestScalar>)>,
        modulo_columns_return: Option<Vec<i128>>,
        get_in_range_column_from_quotient_and_rhs_return: Option<Vec<TestScalar>>,
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
        if let Some(in_range_column) = get_in_range_column_from_quotient_and_rhs_return {
            mock_functionality
                .expect_get_in_range_column_from_quotient_and_rhs()
                .return_const(in_range_column);
        } else {
            mock_functionality
                .expect_get_in_range_column_from_quotient_and_rhs()
                .returning(default_get_in_range_column_from_quotient_and_rhs);
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
        let reduced_matrix = matrix.iter().map(|v| {
            assert!(v[6..(v.len() - 2)].iter().all(|b| *b));
            let mut vec = v[0..6].to_vec();
            vec.extend(v[(v.len() - 2)..v.len()].iter());
            vec
        });
        for row in reduced_matrix {
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
        let reduced_matrix = matrix.iter().map(|v| {
            assert!(v[6..(v.len() - 2)].iter().all(|b| *b));
            let mut vec = v[0..6].to_vec();
            vec.extend(v[(v.len() - 2)..v.len()].iter());
            vec
        });
        for row in reduced_matrix {
            let failing_constraints = get_failing_constraints(&row);
            assert_eq!(failing_constraints, []);
        }
    }

    /// Shifting remainder by a very small amount will only fail the division algorithm
    #[test]
    fn we_can_reject_if_division_algorithm_fails() {
        check_constraints(
            None,
            Some(vec![1i128, -4, 1, -1]),
            None,
            &[8i128, -12, 8, -8],
            &[3i128, 7, -3, -3],
            &[TestableConstraints::DivisionAlgorithm],
        );
    }

    /// When the remainder is 0, shifting the remainder to have the same magnitude as the denominator can trick both the
    /// division algorithm and the remainder bound. However, this should result in the failure of
    /// `TestableConstraints::DenominatorZeroIfRemainderAndDenominatorMagnitudeEqual`
    #[test]
    fn we_can_reject_if_nonzero_remainder_magnitude_equals_denominator_magnitude() {
        check_constraints(
            Some((
                [1, -1].map(TestScalar::from).to_vec(),
                [1, -1].map(TestScalar::from).to_vec(),
            )),
            Some(vec![-4i128, -6]),
            None,
            &[-8i128, -12],
            &[-4i128, 6],
            &[TestableConstraints::DenominatorZeroIfRemainderAndDenominatorMagnitudeEqual],
        );
    }

    /// There is a check to verify that quotient * denominator does not overflow the scalar field.
    /// That check verifies that either quotient or denominator is less than a certain number.
    /// This requires committing to a column of values composed of quotients and denominators.
    /// Providing a value that is not either the quotient or denominator should fail
    /// `TestableContraints::BoundedValueIsQuotientOrDenominator`
    #[test]
    fn we_can_reject_if_committed_in_range_column_is_not_quotient_or_denominator() {
        let quotient = vec![TestScalar::from(1i128 << 126)];
        let rhs_i128 = (1i128 << 126) + 1;
        let overflow: i128 = (TestScalar::from(1i128 << 126) * TestScalar::from(rhs_i128))
            .try_into()
            .unwrap();
        check_constraints(
            Some((quotient.clone(), quotient)),
            Some(vec![rhs_i128 - overflow]),
            Some(vec![TestScalar::ONE]),
            &[rhs_i128],
            &[rhs_i128],
            &[TestableConstraints::BoundedValueIsQuotientOrDenominator],
        );
    }

    /// When the denominator is zero, the quotient must be zero. Lying about quotient in this scenario
    /// is rejected easily by `TestableConstraints::ZeroDenominatorDictatesQuotient`
    #[test]
    fn we_can_reject_if_denominator_is_zero_but_quotient_is_not() {
        let quotient = [10000, -10000, 10000, -10000]
            .map(TestScalar::from)
            .to_vec();
        check_constraints(
            Some((quotient.clone(), quotient)),
            None,
            None,
            &[8i128, -12, -8, 12],
            &[0i128, 0, 0, 0],
            &[TestableConstraints::ZeroDenominatorDictatesQuotient],
        );
    }

    /// If the wrapped version of quotient is completely incorrect and quotient is not -MIN, the constraint
    /// `TestableConstraints::WrappedIsQuotientOrMin` will catch it.
    #[test]
    fn we_can_reject_if_quotient_wrapped_is_incorrect() {
        check_constraints(
            Some((
                [-20i128, -20, 20, 20].map(TestScalar::from).to_vec(),
                [2, -2, 2, -2].map(TestScalar::from).to_vec(),
            )),
            None,
            None,
            &[8i128, -12, -8, 12],
            &[3i128, 5, -3, -5],
            &[TestableConstraints::WrappedIsQuotientOrMin],
        );
    }

    /// If the wrapped version of quotient is completely incorrect and quotient is -MIN, the constraint
    /// `TestableConstraints::WrappedIsMinIfQuotientIsNegativeMin` will catch it.
    #[test]
    fn we_can_reject_if_quotient_is_negative_min_and_quotient_wrapped_is_incorrect() {
        check_constraints(
            Some((
                vec![TestScalar::from(-20)],
                vec![-TestScalar::from(i128::MIN)],
            )),
            None,
            None,
            &[i128::MIN],
            &[-1i128],
            &[TestableConstraints::WrappedIsMinIfQuotientIsNegativeMin],
        );
    }

    /// A malicious prover can try to shift the quotient by 1, adjusting the remainder accordingly,
    /// which still satisfies the division algorithm and stays within the bounds of the +/- denominator.
    /// However, this necessarily requires that the remainder flip sign, violating `TestableConstraints::RemainderSignMatchesNumerator`
    #[test]
    fn we_can_reject_if_remainder_sign_matches_numerator_fails() {
        let quotient = [3i128, -2, -3, 3].map(TestScalar::from).to_vec();
        check_constraints(
            Some((quotient.clone(), quotient)),
            Some(vec![-1i128, 2, -1, 1]),
            None,
            &[8i128, -12, 8, -8],
            &[3i128, 7, -3, -3],
            &[TestableConstraints::RemainderSignMatchesNumerator],
        );
    }

    /// A malicious prover can try to shift the remainder by the rhs, adjusting the quotient accordingly, so that the sign of
    /// the remainder doesn't change. This still satisfies the division algorithm.
    /// However, this necessarily requires that exactly one of `r - b` or `r + b` switches sign, violating `TestableConstraints::RemainderBound`
    #[test]
    fn we_can_reject_if_remainder_bound_fails() {
        let quotient = [1i128, 0, -1, 1].map(TestScalar::from).to_vec();
        check_constraints(
            Some((quotient.clone(), quotient)),
            Some(vec![5i128, -12, 5, -5]),
            None,
            &[8i128, -12, 8, -8],
            &[3i128, 7, -3, -3],
            &[TestableConstraints::RemainderBound],
        );
    }

    #[test]
    fn we_can_reject_if_quotient_and_rhs_are_too_high() {
        let alloc = Bump::new();
        let quotient = vec![TestScalar::from(1i128 << 126)];
        let rhs_i128 = (1i128 << 126) + 1;
        let overflow: i128 = (TestScalar::from(1i128 << 126) * TestScalar::from(rhs_i128))
            .try_into()
            .unwrap();
        let lhs = &[rhs_i128];
        let rhs = &[rhs_i128];
        let mut mock_functionality = MockMockableDivideAndModuloExprFunctionality::new();
        mock_functionality
            .expect_divide_columns()
            .return_const((quotient.clone(), quotient.clone()));
        mock_functionality
            .expect_modulo_columns()
            .return_const(vec![rhs_i128 - overflow]);
        mock_functionality
            .expect_get_in_range_column_from_quotient_and_rhs()
            .return_const(quotient);
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
            1,
            &final_round_builder,
            4,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone() => lhs.inner_product(evaluation_point),
                    rhs_ref.clone() => rhs.inner_product(evaluation_point)
                };
                let err = divide_and_modulo_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval)
                    .unwrap_err();
                assert!(matches!(
                    err,
                    ProofError::VerificationError {
                        error: "Intermediate value out of range"
                    }
                ));
            },
        );
        let matrix = mock_verification_builder.get_identity_results();
        let reduced_matrix = matrix.iter().map(|v| {
            assert!(v[6..(v.len() - 2)].iter().all(|b| *b));
            let mut vec = v[0..6].to_vec();
            vec.extend(v[(v.len() - 2)..v.len()].iter());
            vec
        });
        for row in reduced_matrix {
            let failing_constraints = get_failing_constraints(&row);
            assert_eq!(failing_constraints, []);
        }
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
