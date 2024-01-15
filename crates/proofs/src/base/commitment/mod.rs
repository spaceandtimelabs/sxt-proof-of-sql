//! Types for creation and utilization of cryptographic commitments to proof-of-sql data.

pub use blitzar::compute::{init_backend, init_backend_with_config, BackendConfig};

mod committable_column;
pub use committable_column::CommittableColumn;

mod vec_commitment_ext;
pub use vec_commitment_ext::VecCommitmentExt;

mod column_bounds;
pub use column_bounds::{ColumnBounds, ColumnBoundsMismatch};
