#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;
pub mod symbols;
#[cfg(test)]
mod test;

mod intermediate_ast_utility;
pub use intermediate_ast_utility::{get_ref_tables_from_ast, TableRef};

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all)] pub sql);
