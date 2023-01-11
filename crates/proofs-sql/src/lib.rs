#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;
#[cfg(test)]
mod intermediate_ast_tests;
pub mod symbols;

pub mod select_statement;
pub use select_statement::SelectStatement;

pub mod error;
pub use error::{ParseError, ParseResult};

pub mod resource_id;
pub use resource_id::ResourceId;

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all)] pub sql);
