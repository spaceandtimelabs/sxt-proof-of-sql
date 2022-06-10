mod log;
#[cfg(test)]
mod log_test;
pub use log::{is_pow2, log2_down, log2_up};

mod factorial;
pub use factorial::{scalar_factorial, u128_factorial, u64_factorial};
