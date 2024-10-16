use alloc::string::ToString;
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use serde::{Deserialize, Serialize};
use proof_of_sql_parser::{impl_serde_from_str, Identifier, ResourceId};
use crate::base::resource_id::ResourceId;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
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
    pub fn schema_id(&self) -> Identifier {
        self.resource_id.schema()
    }

    /// Returns the identifier of the table
    #[must_use]
    pub fn table_id(&self) -> Identifier {
        self.resource_id.object_name()
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

impl serde::Serialize for TableRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'d> serde::Deserialize<'d> for TableRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        extern crate alloc;
        let string = alloc::string::String::deserialize(deserializer)?;
        TableRef::from_str(&string).map_err(serde::de::Error::custom)
    }
}