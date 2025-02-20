#![cfg_attr(test, expect(clippy::missing_panics_doc))]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![except(clippy::module_name_repetitions)]

extern crate alloc;

pub mod base;
pub mod proof_primitive;
pub mod sql;
/// Utilities for working with the library
pub mod utils;
