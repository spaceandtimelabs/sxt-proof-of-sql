use crate::base::{
    proof::{Commit, Commitment},
    scalar::IntoScalar,
};
use curve25519_dalek::scalar::Scalar;
use derive_more::{Deref, DerefMut};

/// New-type representing a database column.
#[derive(Clone, Default, Debug, Eq, PartialEq, Deref, DerefMut)]
pub struct Column<T> {
    pub data: Vec<T>,
}

impl<T> Commit for Column<T>
where
    T: IntoScalar + Clone,
{
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        Commitment::from(
            self.iter()
                .map(|d| d.clone().into_scalar())
                .collect::<Vec<Scalar>>()
                .as_slice(),
        )
    }
}

impl<T> From<Vec<T>> for Column<T> {
    fn from(data: Vec<T>) -> Self {
        Column { data }
    }
}
