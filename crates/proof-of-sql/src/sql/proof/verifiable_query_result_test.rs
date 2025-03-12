use super::{
    FinalRoundBuilder, ProofPlan, ProverEvaluate, VerifiableQueryResult, VerificationBuilder,
};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::{bigint, fixed_size_binary, owned_table},
            table_utility::*,
            ColumnField, ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, Table,
            TableEvaluation, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        math::non_negative_i32::NonNegativeI32,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FirstRoundBuilder, QueryData},
};
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub(super) struct EmptyTestQueryExpr {
    pub(super) length: usize,
    pub(super) columns: usize,
}
impl ProverEvaluate for EmptyTestQueryExpr {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let zeros = vec![0_i64; self.length];
        builder.produce_chi_evaluation_length(self.length);
        table_with_row_count(
            (1..=self.columns)
                .map(|i| borrowed_bigint(format!("a{i}").as_str(), zeros.clone(), alloc)),
            self.length,
        )
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let zeros = vec![0_i64; self.length];
        let res: &[_] = alloc.alloc_slice_copy(&zeros);
        let _ = std::iter::repeat_with(|| builder.produce_intermediate_mle(res))
            .take(self.columns)
            .collect::<Vec<_>>();
        table_with_row_count(
            (1..=self.columns)
                .map(|i| borrowed_bigint(format!("a{i}").as_str(), zeros.clone(), alloc)),
            self.length,
        )
    }
}
impl ProofPlan for EmptyTestQueryExpr {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        assert_eq!(
            builder.try_consume_final_round_mle_evaluations(self.columns)?,
            vec![S::ZERO; self.columns]
        );
        Ok(TableEvaluation::new(
            vec![S::ZERO; self.columns],
            builder.try_consume_chi_evaluation()?,
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut fields = Vec::new();
        fields.push(ColumnField::new("a1".into(), ColumnType::BigInt));

        if self.columns == 2 {
            let width = NonNegativeI32::try_from(4).unwrap();
            fields.push(ColumnField::new(
                "a2".into(),
                ColumnType::FixedSizeBinary(width),
            ));
        }
        fields
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {}
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
    }
}

#[test]
fn we_can_verify_queries_on_an_empty_table_with_fixedsizebinary() {
    let expr = EmptyTestQueryExpr {
        columns: 2,
        length: 0,
    };

    let width = NonNegativeI32::try_from(4).unwrap();
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([
            bigint("a1", [0_i64; 0]),
            fixed_size_binary("a2", width, vec![]),
        ]),
        0,
        (),
    );

    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());

    let QueryData { table, .. } = res.verify(&expr, &accessor, &()).unwrap();

    let expected = owned_table([
        bigint("a1", [] as [i64; 0]),
        fixed_size_binary("a2", width, vec![]),
    ]);
    assert_eq!(table, expected);
}

#[test]
fn empty_verification_fails_if_the_result_contains_non_null_members() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };

    let width = NonNegativeI32::try_from(4).unwrap();
    let data = vec![0];

    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([
            bigint("a1", [0_i64; 0]),
            fixed_size_binary("fsb", width, data.clone()),
        ]),
        0,
        (),
    );
    let res = VerifiableQueryResult::<InnerProductProof> {
        result: Some(owned_table([])),
        proof: None,
    };
    assert!(res.verify(&expr, &accessor, &()).is_err());
}
