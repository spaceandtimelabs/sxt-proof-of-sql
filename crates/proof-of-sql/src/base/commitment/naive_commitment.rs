use super::Commitment;
use crate::base::{
    commitment::CommittableColumn,
    scalar::{test_scalar::TestScalar, Scalar},
};
use alloc::{vec, vec::Vec};
use ark_serialize::CanonicalSerialize;
use core::{
    cmp,
    fmt::Debug,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};
use serde::{Deserialize, Serialize};

/// A naive [Commitment] implementation that should only be used for the purpose of unit testing.
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
        let mut new_self = self.clone();
        new_self.sub_assign(rhs);
        new_self
    }
}
impl Neg for NaiveCommitment {
    type Output = NaiveCommitment;

    fn neg(self) -> Self::Output {
        Self(self.0.iter().map(core::ops::Neg::neg).collect())
    }
}

impl SubAssign for NaiveCommitment {
    fn sub_assign(&mut self, rhs: Self) {
        self.add_assign(rhs.neg());
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

impl PartialEq for NaiveCommitment {
    fn eq(&self, other: &Self) -> bool {
        match self.0.len().cmp(&other.0.len()) {
            cmp::Ordering::Less => {
                let mut extended_self = self.0.clone();
                extended_self.extend((self.0.len()..other.0.len()).map(|_i| TestScalar::ZERO));
                extended_self == other.0
            }
            cmp::Ordering::Equal => self.0 == other.0,
            cmp::Ordering::Greater => {
                let mut extended_other = other.0.clone();
                extended_other.extend((other.0.len()..self.0.len()).map(|_i| TestScalar::ZERO));
                extended_other == self.0
            }
        }
    }
}

impl core::ops::Mul<NaiveCommitment> for TestScalar {
    type Output = NaiveCommitment;
    fn mul(self, rhs: NaiveCommitment) -> Self::Output {
        self * &rhs
    }
}
impl core::ops::Mul<TestScalar> for NaiveCommitment {
    type Output = NaiveCommitment;
    fn mul(self, rhs: TestScalar) -> Self::Output {
        &self * rhs
    }
}
impl core::ops::Mul<&NaiveCommitment> for TestScalar {
    type Output = NaiveCommitment;
    fn mul(self, rhs: &NaiveCommitment) -> Self::Output {
        rhs * self
    }
}
impl core::ops::Mul<TestScalar> for &NaiveCommitment {
    type Output = NaiveCommitment;
    fn mul(self, rhs: TestScalar) -> Self::Output {
        NaiveCommitment(self.0.iter().map(|s| rhs * *s).collect())
    }
}

impl Commitment for NaiveCommitment {
    type Scalar = TestScalar;
    type PublicSetup<'a> = ();

    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        committable_columns
            .iter()
            .map(|cc| {
                let mut vectors: Vec<TestScalar> = vec![TestScalar::ZERO; offset];
                let mut existing_scalars: Vec<TestScalar> = match cc {
                    CommittableColumn::Boolean(bool_vec) => {
                        bool_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::Uint8(u8_vec) => {
                        u8_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::TinyInt(tiny_int_vec) => {
                        tiny_int_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::SmallInt(small_int_vec) => small_int_vec
                        .iter()
                        .map(core::convert::Into::into)
                        .collect(),
                    CommittableColumn::Int(int_vec) => {
                        int_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::BigInt(big_int_vec) => {
                        big_int_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::Int128(int_128_vec) => {
                        int_128_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::Decimal75(_, _, u64_vec) => {
                        u64_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::Scalar(scalar_vec) => {
                        scalar_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::VarChar(varchar_vec) => {
                        varchar_vec.iter().map(core::convert::Into::into).collect()
                    }
                    CommittableColumn::TimestampTZ(_, _, i64_vec) => {
                        i64_vec.iter().map(core::convert::Into::into).collect()
                    }
                };
                vectors.append(&mut existing_scalars);
                NaiveCommitment(vectors)
            })
            .collect()
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.0.compressed_size());
        self.0.serialize_compressed(&mut buf).unwrap();
        buf
    }
}

#[allow(clippy::similar_names)]
#[test]
fn we_can_compute_commitments_from_commitable_columns() {
    let column_a = [1i64, 10, -5, 0, 10];
    let column_b = vec![
        [1, 0, 0, 0],
        [2, 0, 0, 0],
        [3, 0, 0, 0],
        [4, 0, 0, 0],
        [5, 0, 0, 0],
    ];
    let column_a_scalars: Vec<TestScalar> =
        column_a.iter().map(core::convert::Into::into).collect();
    let column_b_scalars: Vec<TestScalar> =
        column_b.iter().map(core::convert::Into::into).collect();
    let commitable_column_a = CommittableColumn::BigInt(&column_a);
    let commitable_column_b = CommittableColumn::VarChar(column_b);
    let committable_columns: &[CommittableColumn] = &[commitable_column_a, commitable_column_b];
    let commitments = NaiveCommitment::compute_commitments(committable_columns, 0, &());
    assert_eq!(commitments[0].0, column_a_scalars);
    assert_eq!(commitments[1].0, column_b_scalars);
}

#[test]
fn we_can_compute_commitments_from_commitable_columns_with_offset() {
    let column_a = [0i64, 1, 10, -5, 0, 10];
    let column_a_scalars: Vec<TestScalar> =
        column_a.iter().map(core::convert::Into::into).collect();
    let commitable_column_a = CommittableColumn::BigInt(&column_a[1..]);
    let committable_columns: &[CommittableColumn] = &[commitable_column_a];
    let commitments = NaiveCommitment::compute_commitments(committable_columns, 1, &());
    assert_eq!(commitments[0].0, column_a_scalars);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_get_different_transcript_bytes_from_different_naive_commitments() {
        let commitment1 = NaiveCommitment(vec![TestScalar::from(1), TestScalar::from(2)]);
        let commitment2 = NaiveCommitment(vec![TestScalar::from(3), TestScalar::from(4)]);
        assert_ne!(
            commitment1.to_transcript_bytes(),
            commitment2.to_transcript_bytes()
        );
    }
}
