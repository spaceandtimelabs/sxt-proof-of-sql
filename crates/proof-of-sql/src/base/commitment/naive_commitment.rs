use std::fmt::Debug;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::base::commitment::CommittableColumn;
use crate::base::scalar::test_scalar::TestScalar;
use crate::base::scalar::Scalar;

use super::Commitment;

/// This should only be used for the purpose of unit testing.
#[derive(Clone, Debug, Eq, Default, Serialize, Deserialize)]
pub struct NaiveCommitment(pub Vec<TestScalar>);

impl Add for NaiveCommitment {
    type Output = NaiveCommitment;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_self = self.clone();
        new_self.add_assign(rhs);
        new_self
    }
}

impl Sub for NaiveCommitment {
    type Output = NaiveCommitment;

    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs.neg()
    }
}
impl Neg for NaiveCommitment {
    type Output = NaiveCommitment;

    fn neg(self) -> Self::Output {
        Self(self.0.iter().map(|s| s.neg()).collect())
    }
}

impl SubAssign for NaiveCommitment {
    fn sub_assign(&mut self, rhs: Self) {
        self.add_assign(rhs.neg())
    }
}

impl AddAssign for NaiveCommitment {
    fn add_assign(&mut self, rhs: Self) {
        if self.0.len() < rhs.0.len() {
            self.0
                .extend((self.0.len()..rhs.0.len()).map(|_i| TestScalar::ZERO));
        }
        self.0
            .iter_mut()
            .zip(rhs.0)
            .for_each(|(lhs, rhs)| *lhs += rhs);
    }
}

impl PartialEq for NaiveCommitment{
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() == other.0.len() {
            self.0 == other.0
        }
        else if self.0.len() < other.0.len() {
            let mut extended_self = self.0.clone();
            extended_self.extend((self.0.len()..other.0.len()).map(|_i| TestScalar::ZERO));
            extended_self == other.0
        }
        else{
            let mut extended_other = other.0.clone();
            extended_other.extend((other.0.len()..self.0.len()).map(|_i| TestScalar::ZERO));
            extended_other == self.0
        }
    }
}

impl Commitment for NaiveCommitment {
    type Scalar = TestScalar;
    type PublicSetup<'a> = ();

    fn compute_commitments(
        commitments: &mut [Self],
        committable_columns: &[CommittableColumn],
        offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) {
        let vectors: Vec<Vec<TestScalar>> = committable_columns
            .iter()
            .map(|cc| {
                let mut vectors: Vec<TestScalar> = vec![TestScalar::ZERO; offset];
                let mut existing_scalars: Vec<TestScalar> = cc.into();
                vectors.append(&mut existing_scalars);
                vectors
            })
            .collect();
        commitments.iter_mut().zip(vectors).for_each(|(nc, v)| {
            *nc += NaiveCommitment(v);
        });
    }
}
