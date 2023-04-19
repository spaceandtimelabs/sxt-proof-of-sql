use crate::sql::parse::{ConversionError, ConversionResult};

use indexmap::set::IndexSet;
use proofs_sql::intermediate_ast::ResultColumn;
use proofs_sql::Identifier;
use std::collections::HashMap;

/// A graph that maps column names to their aliases and vice versa.
pub struct ResultColumnAliasGraph {
    /// Maps a column alias to its corresponding name.
    alias_to_names: HashMap<Identifier, Identifier>,
    /// Maps a column name to all of its aliases.
    name_to_aliases: HashMap<Identifier, IndexSet<Identifier>>,
}

impl ResultColumnAliasGraph {
    /// Creates a new `ResultColumnAliasGraph` from the given result columns.
    pub fn new(columns: &[ResultColumn]) -> ConversionResult<Self> {
        let mut alias_to_names = HashMap::<Identifier, Identifier>::new();
        let mut name_to_aliases = HashMap::<Identifier, IndexSet<Identifier>>::new();

        for column in columns {
            let name = column.name;
            let alias = column.alias;

            name_to_aliases.entry(name).or_default().insert(alias);

            // we don't allow duplicate aliases
            if alias_to_names.insert(alias, name).is_some() {
                return Err(ConversionError::DuplicateColumnAlias(
                    alias.name().to_string(),
                ));
            }
        }

        Ok(Self {
            alias_to_names,
            name_to_aliases,
        })
    }

    /// Returns the set of aliases for the given column name.
    pub fn get_name_mapping(&self, name: &Identifier) -> Option<&IndexSet<Identifier>> {
        self.name_to_aliases.get(name)
    }

    /// Returns the associated column name for the given column alias.
    pub fn get_alias_mapping(&self, alias: &Identifier) -> Option<&Identifier> {
        self.alias_to_names.get(alias)
    }
}
