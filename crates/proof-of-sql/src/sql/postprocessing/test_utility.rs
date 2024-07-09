use super::*;
use crate::base::scalar::Scalar;

pub fn slice<S: Scalar>(limit: Option<u64>, offset: Option<i64>) -> OwnedTablePostprocessing<S> {
    OwnedTablePostprocessing::<S>::new_slice(SliceExpr::new(limit, offset))
}
