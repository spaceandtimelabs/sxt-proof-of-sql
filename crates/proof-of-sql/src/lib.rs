#![cfg_attr(test, allow(clippy::missing_panics_doc))]
#![doc = include_str!("../README.md")]
extern crate alloc;

pub mod base;
pub mod proof_primitive;
pub mod sql;

#[cfg(test)]
mod tests;
