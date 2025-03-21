use super::{error::Error, plans::Plan};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, ColumnType, OwnedTable, Table, TableEvaluation, TableRef,
        },
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
use core::str::FromStr;
use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};
use sqlparser::ast::Ident;

#[derive(Debug)]
/// An implementation of `ProofPlan` that allows for EVM compatible serialization.
/// Serialization should be done using bincode with fixint, big-endian encoding in order to be compatible with EVM.
///
/// This is simply a wrapper around a `DynProofPlan`.
pub struct EVMProofPlan {
    inner: DynProofPlan,
}

impl EVMProofPlan {
    /// Create a new `EVMProofPlan` from a `DynProofPlan`.
    #[must_use]
    pub fn new(plan: DynProofPlan) -> Self {
        Self { inner: plan }
    }
    /// Get the inner `DynProofPlan`.
    #[must_use]
    pub fn into_inner(self) -> DynProofPlan {
        self.inner
    }
    /// Get a reference to the inner `DynProofPlan`.
    #[must_use]
    pub fn inner(&self) -> &DynProofPlan {
        &self.inner
    }
}

#[derive(Serialize, Deserialize)]
struct CompactPlan {
    tables: Vec<String>,
    columns: Vec<(usize, String, ColumnType)>,
    plan: Plan,
}

impl Serialize for EVMProofPlan {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let table_refs = self.get_table_references();
        let column_refs = self.get_column_references();

        let plan = Plan::try_from_proof_plan(self.inner(), &table_refs, &column_refs)
            .map_err(serde::ser::Error::custom)?;
        let columns = column_refs
            .into_iter()
            .map(|column_ref| {
                let table_index = table_refs
                    .get_index_of(&column_ref.table_ref())
                    .ok_or(Error::TableNotFound)?;
                Ok((
                    table_index,
                    column_ref.column_id().to_string(),
                    *column_ref.column_type(),
                ))
            })
            .try_collect()
            .map_err(serde::ser::Error::custom::<Error>)?;
        let tables = table_refs.iter().map(ToString::to_string).collect();

        CompactPlan {
            tables,
            columns,
            plan,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EVMProofPlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let compact_plan = CompactPlan::deserialize(deserializer)?;

        let table_refs: IndexSet<TableRef> = compact_plan
            .tables
            .iter()
            .map(|table| TableRef::from_str(table).map_err(|_| Error::ParsingUnsuccessful))
            .try_collect()
            .map_err(serde::de::Error::custom::<Error>)?;
        let table_refs_clone = table_refs.clone();
        let column_refs: IndexSet<ColumnRef> = compact_plan
            .columns
            .iter()
            .map(|(i, ident, column_type)| {
                let table_ref = table_refs_clone
                    .get_index(*i)
                    .cloned()
                    .ok_or(Error::TableNotFound)?;
                Ok(ColumnRef::new(table_ref, Ident::new(ident), *column_type))
            })
            .try_collect()
            .map_err(serde::de::Error::custom::<Error>)?;
        Ok(Self {
            inner: compact_plan
                .plan
                .try_into_proof_plan(&table_refs, &column_refs)
                .map_err(serde::de::Error::custom::<Error>)?,
        })
    }
}

impl ProofPlan for EVMProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        self.inner()
            .verifier_evaluate(builder, accessor, result, chi_eval_map)
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.inner().get_column_result_fields()
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inner().get_column_references()
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.inner().get_table_references()
    }
}
impl ProverEvaluate for EVMProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        self.inner().first_round_evaluate(builder, alloc, table_map)
    }
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        self.inner().final_round_evaluate(builder, alloc, table_map)
    }
}
