use alloc::string::ToString;
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use proof_of_sql_parser::{impl_serde_from_str, sqlparser::object_name_from};
use sqlparser::ast::{Ident, ObjectName};

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableRef {
    object_name: ObjectName,
}

impl TableRef {
    /// Creates a new table reference from a resource id
    #[must_use]
    pub fn new(object_name: ObjectName) -> Self {
        Self {
            object_name: object_name.clone(),
        }
    }

    /// Returns the identifier of the schema
    #[must_use]
    pub fn schema_id(&self) -> Option<Ident> {
        self.object_name.0.get(0).cloned()
    }

    /// Returns the identifier of the table
    #[must_use]
    pub fn table_id(&self) -> Option<Ident> {
        self.object_name.0.get(1).cloned()
    }

    /// Returns the underlying resource id of the table
    #[must_use]
    pub fn object_name(&self) -> ObjectName {
        self.object_name.clone()
    }
}

impl FromStr for TableRef {
    type Err = proof_of_sql_parser::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(object_name_from(s)))
    }
}

impl From<&str> for TableRef {
    fn from(s: &str) -> Self {
        TableRef::new(object_name_from(s))
    }
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.object_name.clone().fmt(f)
    }
}

impl_serde_from_str!(TableRef);
