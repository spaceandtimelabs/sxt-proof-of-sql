mod log;
pub use log::{is_pow2, is_pow2_bytes, log2_down, log2_down_bytes, log2_up, log2_up_bytes};

mod factorial;
pub use factorial::{scalar_factorial, u128_factorial, u64_factorial};

mod decompose;
pub use decompose::{BitDecompose, SignedBitDecompose};
