#![cfg_attr(test, allow(clippy::missing_panics_doc))]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

pub mod base;

pub mod posql_time;
pub mod proof_primitive;
pub mod sql;

#[cfg(test)]
mod tests;
