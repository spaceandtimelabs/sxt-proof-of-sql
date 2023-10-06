use proofs_sql::{impl_serde_from_str, Identifier, ResourceId};
use std::str::FromStr;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct TableRef {
    resource_id: ResourceId,
}

impl TableRef {
    pub fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    pub fn schema_id(&self) -> Identifier {
        self.resource_id.schema()
    }

    pub fn table_id(&self) -> Identifier {
        self.resource_id.object_name()
    }

    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
}

impl FromStr for TableRef {
    type Err = proofs_sql::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

impl std::fmt::Display for TableRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.resource_id.fmt(f)
    }
}

impl_serde_from_str!(TableRef);
