use super::Indexes;
use crate::base::{
    polynomial::compute_evaluation_vector,
    scalar::{Curve25519Scalar, MontScalar},
};
use num_traits::Zero;

#[test]
fn an_empty_sparse_index_slice_is_always_valid() {
    let ix = Indexes::Sparse(vec![]);
    assert!(ix.valid(0));
    assert!(ix.valid(1));
}

#[test]
fn a_single_sparse_index_is_valid_if_within_range() {
    let ix = Indexes::Sparse(vec![0]);
    assert!(!ix.valid(0));
    assert!(ix.valid(1));
}

#[test]
fn multiple_sparse_indexes_are_valid_if_sorted_and_within_range() {
    let ix = Indexes::Sparse(vec![0, 1]);
    assert!(ix.valid(2));
    assert!(!ix.valid(1));

    let ix = Indexes::Sparse(vec![1, 0]);
    assert!(!ix.valid(2));

    let ix = Indexes::Sparse(vec![0, 2, 3, 7]);
    assert!(ix.valid(8));
    assert!(!ix.valid(7));

    let ix = Indexes::Sparse(vec![0, 3, 2, 7]);
    assert!(!ix.valid(8));
}

#[test]
fn repeated_sparse_indexes_are_invalid() {
    let ix = Indexes::Sparse(vec![0, 1, 1]);
    assert!(!ix.valid(2));
}

#[test]
fn dense_indexes_are_valid_if_within_range() {
    let ix = Indexes::Dense(0..0);
    assert!(ix.valid(1));
    assert!(ix.valid(0));

    let ix = Indexes::Dense(0..1);
    assert!(ix.valid(1));
    assert!(!ix.valid(0));

    let ix = Indexes::Dense(0..2);
    assert!(ix.valid(2));
    assert!(!ix.valid(1));

    let ix = Indexes::Dense(1..2);
    assert!(ix.valid(2));
    assert!(!ix.valid(1));

    let ix = Indexes::Dense(2..8);
    assert!(ix.valid(8));
    assert!(!ix.valid(7));
}

#[test]
fn empty_dense_indexes_are_invalid_if_start_and_end_are_not_0() {
    let ix = Indexes::Dense(0..0);
    assert!(ix.valid(10));
    assert!(ix.valid(0));
    let ix = Indexes::Dense(3..3);
    assert!(!ix.valid(10));
    assert!(!ix.valid(0));
    #[allow(clippy::reversed_empty_ranges)]
    let ix = Indexes::Dense(3..2);
    assert!(!ix.valid(10));
    assert!(!ix.valid(0));
}

#[test]
fn we_can_get_the_len_of_indexes() {
    let ix = Indexes::Sparse(vec![0, 1, 1]);
    assert_eq!(ix.len(), 3);

    let ix = Indexes::Sparse(vec![]);
    assert_eq!(ix.len(), 0);

    let ix = Indexes::Dense(0..0);
    assert_eq!(ix.len(), 0);

    let ix = Indexes::Dense(0..1);
    assert_eq!(ix.len(), 1);

    #[allow(clippy::reversed_empty_ranges)]
    let ix = Indexes::Dense(3..2);
    assert_eq!(ix.len(), 0);

    let ix = Indexes::Dense(1..2);
    assert_eq!(ix.len(), 1);

    let ix = Indexes::Dense(2..8);
    assert_eq!(ix.len(), 6);
}

#[test]
fn we_can_get_the_emptiness_of_indexes() {
    let ix = Indexes::Sparse(vec![0, 1, 1]);
    assert!(!ix.is_empty());

    let ix = Indexes::Sparse(vec![]);
    assert!(ix.is_empty());

    let ix = Indexes::Dense(0..0);
    assert!(ix.is_empty());

    let ix = Indexes::Dense(0..1);
    assert!(!ix.is_empty());

    #[allow(clippy::reversed_empty_ranges)]
    let ix = Indexes::Dense(3..2);
    assert!(ix.is_empty());

    let ix = Indexes::Dense(1..2);
    assert!(!ix.is_empty());

    let ix = Indexes::Dense(2..8);
    assert!(!ix.is_empty());
}

#[test]
fn we_can_calculate_the_sum_and_prod_using_iter_for_indexes() {
    let ix = Indexes::Sparse(vec![0, 1, 1]);
    assert_eq!(ix.iter().sum::<u64>(), 2);
    assert_eq!(ix.iter().product::<u64>(), 0);

    let ix = Indexes::Sparse(vec![]);
    assert_eq!(ix.iter().sum::<u64>(), 0);
    assert_eq!(ix.iter().product::<u64>(), 1);

    let ix = Indexes::Sparse(vec![2, 3, 5]);
    assert_eq!(ix.iter().sum::<u64>(), 10);
    assert_eq!(ix.iter().product::<u64>(), 30);

    let ix = Indexes::Dense(0..0);
    assert_eq!(ix.iter().sum::<u64>(), 0);
    assert_eq!(ix.iter().product::<u64>(), 1);

    let ix = Indexes::Dense(0..1);
    assert_eq!(ix.iter().sum::<u64>(), 0);
    assert_eq!(ix.iter().product::<u64>(), 0);

    #[allow(clippy::reversed_empty_ranges)]
    let ix = Indexes::Dense(3..2);
    assert_eq!(ix.iter().sum::<u64>(), 0);
    assert_eq!(ix.iter().product::<u64>(), 1);

    let ix = Indexes::Dense(1..2);
    assert_eq!(ix.iter().sum::<u64>(), 1);
    assert_eq!(ix.iter().product::<u64>(), 1);

    let ix = Indexes::Dense(2..8);
    assert_eq!(ix.iter().sum::<u64>(), 27);
    assert_eq!(ix.iter().product::<u64>(), 5040);
}

#[test]
fn we_can_evaluate_indexes_at_an_evaluation_point() {
    let evaluation_point = [
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(5u64),
        Curve25519Scalar::from(7u64),
    ];
    let mut evaluation_vector = vec![MontScalar::default(); 8];
    compute_evaluation_vector(&mut evaluation_vector, &evaluation_point);

    let ix = Indexes::Sparse(vec![0, 1, 1]);
    assert_eq!(ix.evaluate_at_point(&evaluation_point), None);

    let ix = Indexes::Sparse(vec![]);
    assert_eq!(ix.evaluate_at_point(&evaluation_point), None);

    let ix = Indexes::Sparse(vec![2, 3, 5]);
    assert_eq!(ix.evaluate_at_point(&evaluation_point), None);

    let ix = Indexes::Dense(0..0);
    assert_eq!(ix.evaluate_at_point(&evaluation_point), Some(Zero::zero()));

    let ix = Indexes::Dense(0..1);
    assert_eq!(
        ix.evaluate_at_point(&evaluation_point),
        Some(evaluation_vector[0])
    );

    #[allow(clippy::reversed_empty_ranges)]
    let ix = Indexes::Dense(3..2);
    assert_eq!(ix.evaluate_at_point(&evaluation_point), Some(Zero::zero()));

    let ix = Indexes::Dense(1..2);
    assert_eq!(
        ix.evaluate_at_point(&evaluation_point),
        Some(evaluation_vector[1])
    );

    let ix = Indexes::Dense(2..8);
    assert_eq!(
        ix.evaluate_at_point(&evaluation_point),
        Some(
            evaluation_vector[2]
                + evaluation_vector[3]
                + evaluation_vector[4]
                + evaluation_vector[5]
                + evaluation_vector[6]
                + evaluation_vector[7]
        )
    );
}
