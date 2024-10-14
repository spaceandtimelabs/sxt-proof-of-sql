#[cfg(feature = "arrow")]
use arrow::{error::ArrowError, record_batch::RecordBatch};
use crate::{base::scalar::Scalar, sql::proof::QueryData};

impl<S: Scalar> QueryData<S> {
    #[cfg(all(test, feature = "arrow"))]
    #[must_use]
    pub fn into_record_batch(self) -> RecordBatch {
        self.try_into().unwrap()
    }
}

#[cfg(feature = "arrow")]
impl<S: Scalar> TryFrom<QueryData<S>> for RecordBatch {
    type Error = ArrowError;

    fn try_from(value: QueryData<S>) -> Result<Self, Self::Error> {
        Self::try_from(value.table)
    }
}