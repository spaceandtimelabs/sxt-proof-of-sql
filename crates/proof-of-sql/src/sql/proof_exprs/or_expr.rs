use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
    utils::log,
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable logical OR expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl ProofExpr for OrExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "OrExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let res = Column::Boolean(first_round_evaluate_or(table.num_rows(), alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "OrExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let res = Column::Boolean(final_round_evaluate_or(builder, alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        verifier_evaluate_or(builder, &lhs, &rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "table_length matches lhs and rhs lengths, ensuring no panic occurs"
)]
pub fn first_round_evaluate_or<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &[bool],
    rhs: &[bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] || rhs[i])
}

#[expect(
    clippy::missing_panics_doc,
    reason = "lhs and rhs are guaranteed to have the same length, ensuring no panic occurs"
)]
pub fn final_round_evaluate_or<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    let n = lhs.len();
    assert_eq!(n, rhs.len());

    // lhs_and_rhs
    let lhs_and_rhs: &[_] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
    builder.produce_intermediate_mle(lhs_and_rhs);

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(lhs_and_rhs)]),
            (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
        ],
    );

    // selection
    alloc.alloc_slice_fill_with(n, |i| lhs[i] || rhs[i])
}

pub fn verifier_evaluate_or<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: &S,
    rhs: &S,
) -> Result<S, ProofError> {
    // lhs_and_rhs
    let lhs_and_rhs = builder.try_consume_final_round_mle_evaluation()?;

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        lhs_and_rhs - *lhs * *rhs,
        2,
    )?;

    // selection
    Ok(*lhs + *rhs - lhs_and_rhs)
}
