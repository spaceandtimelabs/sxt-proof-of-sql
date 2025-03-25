use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, NullableColumn, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
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

    #[tracing::instrument(name = "OrExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.result_evaluate(alloc, table).values;
        let rhs_column = self.rhs.result_evaluate(alloc, table).values;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let res = Column::Boolean(result_evaluate_or(table.num_rows(), alloc, lhs, rhs));

        log::log_memory_usage("End");

        NullableColumn::new(res)
    }

    #[tracing::instrument(name = "OrExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.prover_evaluate(builder, alloc, table).values;
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, table).values;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let res = Column::Boolean(prover_evaluate_or(builder, alloc, lhs, rhs));

        log::log_memory_usage("End");

        NullableColumn::new(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<(S, Option<S>), ProofError> {
        let (lhs, _) = self.lhs.verifier_evaluate(builder, accessor, chi_eval)?;
        let (rhs, _) = self.rhs.verifier_evaluate(builder, accessor, chi_eval)?;

        verifier_evaluate_or(builder, &lhs, &rhs).map(|val| (val, None))
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
pub fn result_evaluate_or<'a>(
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
pub fn prover_evaluate_or<'a, S: Scalar>(
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
