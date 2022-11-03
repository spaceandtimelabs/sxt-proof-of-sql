use serde::{Deserialize, Serialize};

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct QueryProof {}

impl QueryProof {}
