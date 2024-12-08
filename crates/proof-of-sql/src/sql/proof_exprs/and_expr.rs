use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable logical AND expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AndExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl AndExpr {
    /// Create logical AND expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl ProofExpr for AndExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        builder.count_subpolynomials(1);
        builder.count_intermediate_mles(1);
        builder.count_degree(3);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "AndExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.result_evaluate(alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.result_evaluate(alloc, table);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        Column::Boolean(alloc.alloc_slice_fill_with(table.num_rows(), |i| lhs[i] && rhs[i]))
    }

    #[tracing::instrument(name = "AndExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column: Column<'a, S> = self.rhs.prover_evaluate(builder, alloc, table);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs: &[bool] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_and_rhs)]),
                (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
            ],
        );
        Column::Boolean(lhs_and_rhs)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<S, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor, one_eval)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor, one_eval)?;

        // lhs_and_rhs
        let lhs_and_rhs = builder.consume_intermediate_mle();

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_and_rhs - lhs * rhs,
        );

        // selection
        Ok(lhs_and_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
