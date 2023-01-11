#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;
#[cfg(test)]
mod intermediate_ast_tests;
pub mod symbols;

mod intermediate_ast_utility;
pub use intermediate_ast_utility::{get_ref_tables_from_ast, TableRef};

pub mod error;
pub use error::{ParseError, ParseResult};

pub mod resource_id;
pub use resource_id::ResourceId;

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all)] pub sql);
