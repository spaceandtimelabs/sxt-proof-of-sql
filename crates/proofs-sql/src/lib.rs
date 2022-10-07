#[macro_use]
extern crate lalrpop_util;

pub mod intermediate_ast;
pub mod symbols;
#[cfg(test)]
mod test;

// lalrpop-generated code is not clippy-compliant
lalrpop_mod!(#[allow(clippy::all)] pub sql);
