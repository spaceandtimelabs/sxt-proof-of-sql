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
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivideAndModuloExpr {
    pub lhs: Box<DynProofExpr>,
    pub rhs: Box<DynProofExpr>,
}

const SQRT_MIN_I128: u64 = 13_043_817_825_332_782_212;

impl DivideAndModuloExpr {
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }

    #[allow(clippy::missing_panics_doc)]
    fn min_scalar<S: Scalar>(&self) -> S {
        self.lhs.data_type().min_scalar::<S>().unwrap()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn data_type(&self) -> ColumnType {
        try_divide_modulo_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to divide/modulo column types")
            .0
    }

    #[allow(clippy::too_many_lines)]
    pub fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> (Column<'a, S>, Column<'a, S>) {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.prover_evaluate(builder, alloc, table);

        let (quotient_wrapped, quotient) = divide_columns(&lhs_column, &rhs_column, alloc);
        let remainder = modulo_columns(&lhs_column, &rhs_column, alloc);
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
        let min_sqrt_scalar = -S::from(SQRT_MIN_I128);
        let in_range_q_or_b = alloc.alloc_slice_fill_with(quotient.len(), |_i| S::ZERO);
        for (res, (q, b)) in in_range_q_or_b
            .iter_mut()
            .zip(quotient.iter().copied().zip(rhs_as_scalars.clone()))
        {
            // We do or rather than and here because scalars wrap negative values, so only one can be true at a time
            let in_range_value = if q > min_sqrt_scalar || q < -min_sqrt_scalar {
                q
            } else {
                b
            };
            *res = in_range_value;
        }
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
        // (q′ − q) * (q′ − MIN) = 0
        // Ensures that either q = q' or q' = MIN
        // Simplifies to
        // q' * q' + MIN * q - q * q' - MIN * q'

        let min_scalar = self.min_scalar();
        let min_column =
            Column::Scalar(alloc.alloc_slice_fill_with(quotient.len(), |_i| min_scalar));

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    S::one(),
                    vec![Box::new(quotient_wrapped), Box::new(quotient_wrapped)],
                ),
                (S::one(), vec![Box::new(quotient), Box::new(min_column)]),
                (
                    -S::one(),
                    vec![Box::new(quotient_wrapped), Box::new(quotient)],
                ),
                (
                    -S::one(),
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
            Column::Scalar(alloc.alloc_slice_fill_with(quotient.len(), |_i| -min_sqrt_scalar));
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

        log::log_memory_usage("End");

        (quotient_wrapped, remainder)
    }

    pub fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
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

        // b * t = q
        let q_div_b = builder.try_consume_final_round_mle_evaluation()?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            rhs * q_div_b - quotient,
            2,
        )?;

        // (q′ − q) * (q′ − MIN) = 0
        let min_eval = self.min_scalar::<S>() * one_eval;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (quotient_wrapped - quotient) * (quotient_wrapped - min_eval),
            2,
        )?;

        // (q' - MIN) * (q + MIN) * v - (q' - MIN) = 0
        let v = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            (quotient_wrapped - min_eval) * (quotient + min_eval) * v - quotient_wrapped + min_eval,
            3,
        )?;

        // sign(sqrt(-min) + s) = 1
        // sign(sqrt(-min) - s) = 1
        let min_sqrt_eval = S::from(SQRT_MIN_I128) * one_eval;
        let sqrt_min_plus_s = verifier_evaluate_sign(builder, min_sqrt_eval + s, one_eval, 128)?;
        let sqrt_min_less_s = verifier_evaluate_sign(builder, min_sqrt_eval - s, one_eval, 128)?;

        if sqrt_min_plus_s != S::ZERO || sqrt_min_less_s != S::ZERO {
            return Err(ProofError::VerificationError {
                error: "Intermediate value out of range",
            });
        }

        // MIN < q < -MIN
        // We need at least 129 to allow for -MIN
        verifier_evaluate_sign(builder, quotient, one_eval, 129)?;

        // sign(a) * r = sign(r) * r and sign(r - b) * b + sign(r + b) * b = b
        let lhs_sign = verifier_evaluate_sign(builder, lhs, one_eval, 128)?;
        let remainder_sign = verifier_evaluate_sign(builder, remainder, one_eval, 128)?;

        let remainder_and_rhs_difference_sign =
            verifier_evaluate_sign(builder, remainder - rhs, one_eval, 129)?;
        let remainder_and_rhs_added_sign =
            verifier_evaluate_sign(builder, remainder + rhs, one_eval, 129)?;

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

        Ok((quotient, remainder))
    }

    pub fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
