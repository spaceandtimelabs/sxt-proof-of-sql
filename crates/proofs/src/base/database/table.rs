use proofs_sql::ResourceId;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableRef {
    resource_id: ResourceId,
}

impl TableRef {
    pub fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    pub fn schema(&self) -> &str {
        self.resource_id.schema().name()
    }

    pub fn table_name(&self) -> &str {
        self.resource_id.object_name().name()
    }
}
