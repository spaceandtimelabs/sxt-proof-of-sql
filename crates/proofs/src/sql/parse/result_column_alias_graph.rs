use crate::sql::ast::FilterResultExpr;

use indexmap::set::IndexSet;
use proofs_sql::Identifier;
use std::collections::HashMap;

/// A graph that maps column names to their aliases and vice versa.
pub struct ResultColumnAliasGraph {
    /// Maps a column alias to all of its names.
    alias_to_names: HashMap<Identifier, IndexSet<Identifier>>,
    /// Maps a column name to all of its aliases.
    name_to_aliases: HashMap<Identifier, IndexSet<Identifier>>,
}

impl ResultColumnAliasGraph {
    /// Creates a new `ResultColumnAliasGraph` from the given result columns.
    pub fn new(columns: &[FilterResultExpr]) -> Self {
        let mut alias_to_names = HashMap::<Identifier, IndexSet<Identifier>>::new();
        let mut name_to_aliases = HashMap::<Identifier, IndexSet<Identifier>>::new();

        for column in columns {
            let alias = column.get_column_alias_name();
            let name = column.get_column_reference().column_id();

            alias_to_names.entry(alias).or_default().insert(name);
            name_to_aliases.entry(name).or_default().insert(alias);
        }

        Self {
            alias_to_names,
            name_to_aliases,
        }
    }

    /// Returns the set of aliases for the given column name.
    pub fn get_name_mapping(&self, name: &Identifier) -> Option<&IndexSet<Identifier>> {
        self.name_to_aliases.get(name)
    }

    /// Returns the set of names for the given column alias.
    pub fn get_alias_mapping(&self, alias: &Identifier) -> Option<&IndexSet<Identifier>> {
        self.alias_to_names.get(alias)
    }
}
