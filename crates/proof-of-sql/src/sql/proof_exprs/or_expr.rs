use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnNullability, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable logical OR expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrExpr<C: Commitment> {
    lhs: Box<DynProofExpr<C>>,
    rhs: Box<DynProofExpr<C>>,
}

impl<C: Commitment> OrExpr<C> {
    /// Create logical OR expression
    pub fn new(lhs: Box<DynProofExpr<C>>, rhs: Box<DynProofExpr<C>>) -> Self {
        Self { lhs, rhs }
    }
}

impl<C: Commitment> ProofExpr<C> for OrExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_or(builder);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean(ColumnNullability::NotNullable)
    }

    #[tracing::instrument(name = "OrExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> =
            self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> =
            self.rhs.result_evaluate(table_length, alloc, accessor);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        Column::Boolean(
            ColumnNullability::NotNullable,
            result_evaluate_or(table_length, alloc, lhs, rhs),
        )
    }

    #[tracing::instrument(name = "OrExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> = self.rhs.prover_evaluate(builder, alloc, accessor);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        Column::Boolean(
            ColumnNullability::NotNullable,
            prover_evaluate_or(builder, alloc, lhs, rhs),
        )
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor)?;

        Ok(verifier_evaluate_or(builder, &lhs, &rhs))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[allow(
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

#[allow(
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

pub fn verifier_evaluate_or<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    lhs: &C::Scalar,
    rhs: &C::Scalar,
) -> C::Scalar {
    // lhs_and_rhs
    let lhs_and_rhs = builder.consume_intermediate_mle();

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.produce_sumcheck_subpolynomial_evaluation(
        &SumcheckSubpolynomialType::Identity,
        lhs_and_rhs - *lhs * *rhs,
    );

    // selection
    *lhs + *rhs - lhs_and_rhs
}

pub fn count_or(builder: &mut CountBuilder) {
    builder.count_subpolynomials(1);
    builder.count_intermediate_mles(1);
    builder.count_degree(3);
}
