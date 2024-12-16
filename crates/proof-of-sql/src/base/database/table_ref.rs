use alloc::string::ToString;
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use proof_of_sql_parser::{impl_serde_from_str, ResourceId};
use sqlparser::ast::Ident;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct TableRef {
    resource_id: ResourceId,
}

impl TableRef {
    /// Creates a new table reference from a resource id
    #[must_use]
    pub fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    /// Returns the identifier of the schema
    #[must_use]
    pub fn schema_id(&self) -> Ident {
        self.resource_id.schema().into()
    }

    /// Returns the identifier of the table
    #[must_use]
    pub fn table_id(&self) -> Ident {
        self.resource_id.object_name().into()
    }

    /// Returns the underlying resource id of the table
    #[must_use]
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
}

impl FromStr for TableRef {
    type Err = proof_of_sql_parser::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.resource_id.fmt(f)
    }
}

impl_serde_from_str!(TableRef);
