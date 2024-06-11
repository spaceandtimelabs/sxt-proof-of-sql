use super::record_batch_accessor::RecordBatchAccessor;
use arrow::{datatypes::Schema, record_batch::RecordBatch};
use arrow_csv::{ReaderBuilder, WriterBuilder};
use proofs::base::{
    database::{Column, ColumnRef, DataAccessor, MetadataAccessor, SchemaAccessor, TableRef},
    scalar::Scalar,
};
use std::{
    error::Error,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::Arc,
};

fn write_record_batch_to_csv(batch: &RecordBatch, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new().build(File::create(path)?);
    writer.write(batch)?;
    Ok(())
}
pub fn read_record_batch_from_csv(
    schema: Schema,
    path: &Path,
) -> Result<RecordBatch, Box<dyn Error>> {
    let mut csv = ReaderBuilder::new(Arc::new(schema))
        .has_header(true)
        .build(File::open(path)?)?;
    let batch = csv.next().ok_or("Empty table.")??;
    Ok(batch)
}
fn append_record_batch_to_csv(batch: &RecordBatch, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .build(OpenOptions::new().append(true).open(path)?);
    writer.write(batch)?;
    Ok(())
}

pub struct CsvDataAccessor {
    base_path: PathBuf,
    inner: RecordBatchAccessor,
}

impl CsvDataAccessor {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            inner: Default::default(),
        }
    }
    pub fn load_table(
        &mut self,
        table_ref: TableRef,
        schema: Schema,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.get_table_path(&table_ref);
        let batch = super::read_record_batch_from_csv(schema, &path)?;
        self.inner.insert_table(table_ref, batch);
        Ok(())
    }
    fn get_table_path(&self, table_ref: &TableRef) -> PathBuf {
        self.base_path.join(format!("{}.csv", table_ref))
    }
    pub fn write_table(
        &self,
        table_ref: &TableRef,
        batch: &RecordBatch,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.get_table_path(table_ref);
        write_record_batch_to_csv(batch, &path)?;
        Ok(())
    }
    pub fn append_batch(
        &self,
        table_ref: &TableRef,
        batch: &RecordBatch,
    ) -> Result<(), Box<dyn Error>> {
        let path = self.get_table_path(table_ref);
        append_record_batch_to_csv(batch, &path)?;
        Ok(())
    }
}
impl<S: Scalar> DataAccessor<S> for CsvDataAccessor {
    fn get_column(&self, column: ColumnRef) -> Column<S> {
        self.inner.get_column(column)
    }
}
impl MetadataAccessor for CsvDataAccessor {
    fn get_length(&self, table_ref: TableRef) -> usize {
        self.inner.get_length(table_ref)
    }
    fn get_offset(&self, table_ref: TableRef) -> usize {
        self.inner.get_offset(table_ref)
    }
}
impl SchemaAccessor for CsvDataAccessor {
    fn lookup_column(
        &self,
        table_ref: TableRef,
        column_id: proofs_sql::Identifier,
    ) -> Option<proofs::base::database::ColumnType> {
        self.inner.lookup_column(table_ref, column_id)
    }
    fn lookup_schema(
        &self,
        table_ref: TableRef,
    ) -> Vec<(proofs_sql::Identifier, proofs::base::database::ColumnType)> {
        self.inner.lookup_schema(table_ref)
    }
}
