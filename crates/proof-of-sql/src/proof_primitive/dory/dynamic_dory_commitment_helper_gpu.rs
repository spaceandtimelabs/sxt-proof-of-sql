use super::{
    DynamicDoryCommitment,
    ProverSetup,
};
use crate::base::commitment::CommittableColumn;

pub(super) fn compute_dynamic_dory_commitments(
    _committable_columns: &[CommittableColumn],
    _offset: usize,
    _setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    todo!()
}
