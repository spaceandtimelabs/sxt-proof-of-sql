use super::{error::Error, plans::Plan};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_plans::DynProofPlan,
    },
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bumpalo::Bump;
use serde::{Serialize, Serializer};

#[derive(Debug)]
/// An implementation of `ProofPlan` that allows for EVM compatible serialization.
/// Serialization should be done using bincode with fixint, big-endian encoding in order to be compatible with EVM.
///
/// This is simply a wrapper around a `DynProofPlan`.
pub struct EVMProofPlan(pub DynProofPlan);

impl Serialize for EVMProofPlan {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct CompactPlan {
            tables: Vec<String>,
            columns: Vec<(usize, String)>,
            plan: Plan,
        }

        let table_refs = self.get_table_references();
        let column_refs = self.get_column_references();

        let plan = Plan::try_from_proof_plan(&self.0, &table_refs, &column_refs)
            .map_err(serde::ser::Error::custom)?;
        let columns = column_refs
            .into_iter()
            .map(|column_ref| {
                table_refs
                    .get_index_of(&column_ref.table_ref())
                    .map(|table_index| (table_index, column_ref.column_id().to_string()))
            })
            .collect::<Option<_>>()
            .ok_or(Error::TableNotFound)
            .map_err(serde::ser::Error::custom)?;
        let tables = table_refs.iter().map(ToString::to_string).collect();

        CompactPlan {
            tables,
            columns,
            plan,
        }
        .serialize(serializer)
    }
}

impl ProofPlan for EVMProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        self.0
            .verifier_evaluate(builder, accessor, result, one_eval_map)
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.0.get_column_result_fields()
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.0.get_column_references()
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.0.get_table_references()
    }
}
impl ProverEvaluate for EVMProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        self.0.first_round_evaluate(builder, alloc, table_map)
    }
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        self.0.final_round_evaluate(builder, alloc, table_map)
    }
}
