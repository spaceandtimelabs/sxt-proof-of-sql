use proofs::base::{
    commitment::{Commitment, QueryCommitments, TableCommitment},
    database::{CommitmentAccessor, MetadataAccessor, SchemaAccessor, TableRef},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};
pub struct CommitAccessor<C: Commitment> {
    base_path: PathBuf,
    inner: QueryCommitments<C>,
}
impl<C: Commitment + Serialize + for<'a> Deserialize<'a>> CommitAccessor<C> {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            inner: Default::default(),
        }
    }
    pub fn write_commit(
        &self,
        table_ref: &TableRef,
        commit: &TableCommitment<C>,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join(format!("{}.commit", table_ref));
        fs::write(path, postcard::to_allocvec(commit)?)?;
        Ok(())
    }
    pub fn load_commit(&mut self, table_ref: TableRef) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join(format!("{}.commit", table_ref));
        let commit = postcard::from_bytes(&fs::read(path)?)?;
        self.inner.insert(table_ref, commit);
        Ok(())
    }
    pub fn get_commit(&self, table_ref: &TableRef) -> Option<&TableCommitment<C>> {
        self.inner.get(table_ref)
    }
}

impl<C: Commitment> CommitmentAccessor<C> for CommitAccessor<C> {
    fn get_commitment(&self, column: proofs::base::database::ColumnRef) -> C {
        self.inner.get_commitment(column)
    }
}
impl<C: Commitment> MetadataAccessor for CommitAccessor<C> {
    fn get_length(&self, table_ref: proofs::base::database::TableRef) -> usize {
        self.inner.get_length(table_ref)
    }

    fn get_offset(&self, table_ref: proofs::base::database::TableRef) -> usize {
        self.inner.get_offset(table_ref)
    }
}
impl<C: Commitment> SchemaAccessor for CommitAccessor<C> {
    fn lookup_column(
        &self,
        table_ref: proofs::base::database::TableRef,
        column_id: proofs_sql::Identifier,
    ) -> Option<proofs::base::database::ColumnType> {
        self.inner.lookup_column(table_ref, column_id)
    }

    fn lookup_schema(
        &self,
        table_ref: proofs::base::database::TableRef,
    ) -> Vec<(proofs_sql::Identifier, proofs::base::database::ColumnType)> {
        self.inner.lookup_schema(table_ref)
    }
}
