#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod base;
#[cfg(feature = "std")]
pub mod proof_primitive;
#[cfg(feature = "std")]
pub mod sql;

#[cfg(test)]
#[cfg(feature = "std")]
mod tests;
