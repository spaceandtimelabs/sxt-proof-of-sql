#![doc = include_str!("../README.md")]
extern crate alloc;

/// Module for handling an intermediate decimal type received from the lexer.
pub mod intermediate_decimal;
/// Module for handling an intermediate timestamp type received from the lexer.
pub mod posql_time;
#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;

#[cfg(test)]
mod intermediate_ast_tests;

/// Shortcuts to construct intermediate AST nodes.
pub mod utility;

/// TODO: add docs
pub(crate) mod select_statement;
pub use select_statement::SelectStatement;

/// Error definitions for proof-of-sql-parser
pub mod error;
pub use error::ParseError;
pub(crate) use error::ParseResult;

/// TODO: add docs
pub(crate) mod identifier;
pub use identifier::Identifier;

pub mod resource_id;
pub use resource_id::ResourceId;

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all, missing_docs, clippy::missing_docs_in_private_items)] pub sql);

/// Implement Deserialize through `FromStr` to avoid invalid identifiers.
#[macro_export]
macro_rules! impl_serde_from_str {
    ($type:ty) => {
        impl serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
        impl<'d> serde::Deserialize<'d> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'d>,
            {
                extern crate alloc;
                let string = alloc::string::String::deserialize(deserializer)?;
                <$type>::from_str(&string).map_err(serde::de::Error::custom)
            }
        }
    };
}
