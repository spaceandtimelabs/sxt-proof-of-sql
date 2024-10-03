#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::missing_panics_doc)] // Fixed in Issue #163
extern crate alloc;

pub mod base;
pub mod proof_primitive;
pub mod sql;

#[cfg(test)]
mod tests;
