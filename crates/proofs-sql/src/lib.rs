//! TODO: add docs

pub mod decimal_unknown;
#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;

#[cfg(test)]
mod intermediate_ast_tests;

#[cfg(test)]
pub mod test_utility;

pub mod select_statement;
pub use select_statement::SelectStatement;

pub mod error;
pub use error::{ParseError, ParseResult};

pub mod identifier;
pub use identifier::Identifier;

pub mod resource_id;
pub use resource_id::ResourceId;

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all, missing_docs)] pub sql);

/// Implement Deserialize through FromStr to avoid invalid identifiers.
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
                let string = String::deserialize(deserializer)?;
                <$type>::from_str(&string).map_err(serde::de::Error::custom)
            }
        }
    };
}
