//! This file is used to generate the parser from the grammar file.
extern crate lalrpop;

fn main() {
    lalrpop::process_root().unwrap();
}
