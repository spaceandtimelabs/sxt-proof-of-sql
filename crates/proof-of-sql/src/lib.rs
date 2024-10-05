#![cfg_attr(test, allow(clippy::missing_panics_doc))]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::module_name_repetitions)]
#![deny(
    clippy::doc_markdown,
    clippy::match_same_arms,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::semicolon_if_nothing_returned,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::must_use_candidate,
    clippy::range_plus_one
)]

extern crate alloc;

pub mod base;
pub mod proof_primitive;
pub mod sql;

#[cfg(test)]
mod tests;
