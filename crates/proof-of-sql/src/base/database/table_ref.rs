use crate::base::database::error::ParseError;
use alloc::{string::ToString, vec::Vec};
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use indexmap::Equivalent;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableRef {
    schema_name: Option<Ident>,
    table_name: Ident,
}

impl TableRef {
    /// Creates a new table reference from schema and table names.
    /// If the schema name is empty or None, only the table name is used.
    #[must_use]
    pub fn new(schema_name: impl AsRef<str>, table_name: impl AsRef<str>) -> Self {
        let schema = schema_name.as_ref();
        let table = table_name.as_ref();

        Self {
            schema_name: if schema.is_empty() {
                None
            } else {
                Some(Ident::new(schema.to_string()))
            },
            table_name: Ident::new(table.to_string()),
        }
    }

    /// Returns the identifier of the schema
    /// # Panics
    #[must_use]
    pub fn schema_id(&self) -> Option<&Ident> {
        self.schema_name.as_ref()
    }

    /// Returns the identifier of the table
    /// # Panics
    #[must_use]
    pub fn table_id(&self) -> &Ident {
        &self.table_name
    }

    /// Creates a new table reference from an optional schema and table name.
    #[must_use]
    pub fn from_names(schema_name: Option<&str>, table_name: &str) -> Self {
        Self {
            schema_name: schema_name.map(|s| Ident::new(s.to_string())),
            table_name: Ident::new(table_name.to_string()),
        }
    }

    /// Creates a `TableRef` directly from `Option<Ident>` for schema and `Ident` for table.
    #[must_use]
    pub fn from_idents(schema_name: Option<Ident>, table_name: Ident) -> Self {
        Self {
            schema_name,
            table_name,
        }
    }

    /// Creates a `TableRef` from a slice of string components.
    pub fn from_strs<S: AsRef<str>>(components: &[S]) -> Result<Self, ParseError> {
        match components.len() {
            1 => Ok(Self::from_names(None, components[0].as_ref())),
            2 => Ok(Self::from_names(
                Some(components[0].as_ref()),
                components[1].as_ref(),
            )),
            _ => Err(ParseError::InvalidTableReference {
                table_reference: components
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>()
                    .join(","),
            }),
        }
    }
}

/// Creates a `TableRef` from a dot-separated string.
impl TryFrom<&str> for TableRef {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let components: Vec<_> = s.split('.').map(ToString::to_string).collect();
        match components.len() {
            1 => Ok(Self::from_names(None, &components[0])),
            2 => Ok(Self::from_names(Some(&components[0]), &components[1])),
            _ => Err(ParseError::InvalidTableReference {
                table_reference: s.to_string(),
            }),
        }
    }
}

impl FromStr for TableRef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Equivalent<TableRef> for &TableRef {
    fn equivalent(&self, key: &TableRef) -> bool {
        self.schema_name == key.schema_name && self.table_name == key.table_name
    }
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(schema) = &self.schema_name {
            write!(f, "{}.{}", schema.value, self.table_name.value)
        } else {
            write!(f, "{}", self.table_name.value)
        }
    }
}

impl Serialize for TableRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'d> Deserialize<'d> for TableRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let string = alloc::string::String::deserialize(deserializer)?;
        TableRef::from_str(&string).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_ref_new_with_and_without_schema() {
        let with_schema = TableRef::new("schema_name", "table_name");
        assert_eq!(with_schema.schema_id(), Some(&Ident::new("schema_name")));
        assert_eq!(with_schema.table_id(), &Ident::new("table_name"));

        let without_schema = TableRef::new("", "table_name");
        assert_eq!(without_schema.schema_id(), None);
        assert_eq!(without_schema.table_id(), &Ident::new("table_name"));
    }

    #[test]
    fn test_table_ref_from_names() {
        let ref_names = TableRef::from_names(Some("schema_name"), "table_name");
        assert_eq!(ref_names.schema_id(), Some(&Ident::new("schema_name")));
        assert_eq!(ref_names.table_id(), &Ident::new("table_name"));
    }

    #[test]
    fn test_table_ref_from_idents() {
        let schema = Ident::new("schema_name");
        let table = Ident::new("table_name");
        let ref_idents = TableRef::from_idents(Some(schema.clone()), table.clone());
        assert_eq!(ref_idents.schema_id(), Some(&schema));
        assert_eq!(ref_idents.table_id(), &table);
    }

    #[test]
    fn test_table_ref_from_strs() {
        let single_component = ["table_name"];
        let ref_single = TableRef::from_strs(&single_component).unwrap();
        assert_eq!(ref_single.schema_id(), None);
        assert_eq!(ref_single.table_id(), &Ident::new("table_name"));

        let double_component = ["schema_name", "table_name"];
        let ref_double = TableRef::from_strs(&double_component).unwrap();
        assert_eq!(ref_double.schema_id(), Some(&Ident::new("schema_name")));
        assert_eq!(ref_double.table_id(), &Ident::new("table_name"));

        let triple_component = ["db_name", "schema_name", "table_name"];
        let result = TableRef::from_strs(&triple_component);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_ref_try_from_str() {
        let single = TableRef::try_from("table_name").unwrap();
        assert_eq!(single.schema_id(), None);
        assert_eq!(single.table_id(), &Ident::new("table_name"));

        let double = TableRef::try_from("schema_name.table_name").unwrap();
        assert_eq!(double.schema_id(), Some(&Ident::new("schema_name")));
        assert_eq!(double.table_id(), &Ident::new("table_name"));

        let triple = TableRef::try_from("db_name.schema_name.table_name");
        assert!(triple.is_err());
    }

    #[test]
    fn test_table_ref_from_str_trait() {
        let table_ref: TableRef = "schema_name.table_name".parse().unwrap();
        assert_eq!(table_ref.schema_id(), Some(&Ident::new("schema_name")));
        assert_eq!(table_ref.table_id(), &Ident::new("table_name"));
    }

    #[test]
    fn test_table_ref_equivalent() {
        let ref1 = TableRef::new("schema_name", "table_name");
        let ref2 = TableRef::new("schema_name", "table_name");
        let ref3 = TableRef::new("other_schema", "table_name");

        assert!(ref1.equivalent(&ref2));
        assert!(!ref1.equivalent(&ref3));
    }

    #[test]
    fn test_table_ref_display() {
        let with_schema = TableRef::new("schema_name", "table_name");
        assert_eq!(with_schema.to_string(), "schema_name.table_name");

        let without_schema = TableRef::new("", "table_name");
        assert_eq!(without_schema.to_string(), "table_name");
    }

    #[test]
    fn test_table_ref_serde_roundtrip() {
        let table_ref = TableRef::new("schema_name", "table_name");
        let serialized = serde_json::to_string(&table_ref).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(table_ref, deserialized);
    }
}
